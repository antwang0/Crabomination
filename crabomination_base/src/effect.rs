//! Primitive, data-driven effect system: a small set of composable algebras
//! that replaced the old per-effect `SpellEffect`/`TriggerCondition`/
//! `EffectCondition`/`StaticAbilityTemplate` enums.
//!
//! * [`Selector`] — lazy reference to game objects or players, resolved at
//!   effect-time.
//! * [`Value`]    — numeric expression (counts, life totals, X).
//! * [`Predicate`]— game-state boolean (for conditional effects).
//! * [`Effect`]   — the unified instruction tree executed by the resolver.
//! * [`EventSpec`]— structural trigger filter over the [`GameEvent`] stream.
//! * [`StaticEffect`] — description of a static ability's continuous effect.
//!
//! A new card is usually a tree of these primitives, not an engine change.

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
    /// The owner of the card currently being moved (resolved per card by
    /// `place_card_in_dest`), so one `Move` sends each card to its own owner
    /// (Evacuation). `None` outside a placement context.
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
    /// Every opponent other than the trigger's own player — "each other
    /// opponent" on a trigger whose player is one of them (Grenzo's Ruffians).
    EachOpponentExceptTriggerer,
    /// The player who triggered the event (for triggered abilities).
    Triggerer,
    /// The seat that caused the firing event (the trigger's stamped actor,
    /// `GameState::trigger_event_player_scratch`): "that spell or ability's
    /// controller" (Lava Runner), where `Triggerer` gives the target's controller.
    TriggerEventPlayer,
    /// A concrete seat index, used internally to flatten selector-based refs
    /// (`OwnerOf(Selector)`) before crossing a context boundary, where the card
    /// lookup could go stale once the card moves.
    Seat(usize),
    /// The player (or planeswalker controller) the source creature is attacking;
    /// `None` when it isn't attacking. Goblin Guide.
    DefendingPlayer,
    /// Controller of the source that last dealt combat damage to the inner
    /// permanent (`CardInstance.combat_damager_controller`, which outlives the
    /// `block_map` teardown). Souls of the Faultless.
    CombatDamagerController(Box<Selector>),
    /// CR 506.2 — the player (or planeswalker controller) `selector`'s creature
    /// is attacking; per-attacker `DefendingPlayer` for fan-outs (Jurin).
    DefendingPlayerOf(Box<Selector>),
    /// Controller of the last source that dealt damage (combat or not) to the
    /// inner permanent this turn (`CardInstance.damaged_by_this_turn`). Belltower
    /// Sphinx.
    LastDamagerControllerOf(Box<Selector>),
    /// The player with the lowest life total, ties broken by the resolving
    /// controller's choice (auto-picks the earliest seat). Loxodon Peacekeeper.
    LowestLife,
    /// The player with the highest life total, ties broken toward the earliest
    /// seat. Wild Dogs' "the player with the most life gains control".
    HighestLife,
    /// An opponent of the controller with the most life among them, ties
    /// toward the earliest seat — Galactus, Devourer of Worlds.
    HighestLifeOpponent,
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
    /// Every living player tied for the fewest permanents matching the filter
    /// they control, in APNAP order ("each player who controls the fewest
    /// creatures" — Gor Muldrak, Amphinologist).
    EachWithFewestControlled(Box<SelectionRequirement>),
    /// The player the source remembered (`CardInstance.chosen_player`, stamped
    /// by [`Effect::RememberPlayerOnSource`]) — "that player" on a later
    /// trigger. Soul Scourge, Laquatus's Champion.
    ChosenPlayerOfSource,
    /// "Each player this creature attacked this turn" (Angel of Destiny) —
    /// the source's `attacked_players`, alive seats in APNAP order; nobody
    /// once the source has left the battlefield.
    EachPlayerSourceAttackedThisTurn,
    /// "A player you're attacking" (CR 506.2): each live seat a creature the
    /// controller controls is attacking directly, APNAP order; nobody outside
    /// combat. Undercellar Sweep's "if you or a player you're attacking…".
    EachPlayerYouAreAttacking,
    /// "Choose an opponent **at random**" (Raving Dead) — a live opponent
    /// drawn from the game's RNG. A fresh draw each time it is resolved, so
    /// it belongs in a one-shot slot such as `Effect::RememberPlayerOnSource`.
    RandomOpponent,
    /// "Attach this Aura to **another** one of your opponents chosen at
    /// random" (Maddening Hex): `RandomOpponent` drawn from the opponents other
    /// than the player the source is currently attached to. With no other
    /// opponent left it resolves to nobody, so the Aura stays put.
    RandomOtherOpponentThanEnchanted,
    /// "Choose a player at random" (Wildfire Devils) — any living seat, the
    /// controller included, drawn from the game's RNG on each resolution.
    RandomPlayer,
    /// "The player to your right" (Bucknard's Everfull Purse) — the nearest
    /// living seat one step *against* turn order (CR 101.4: play passes to
    /// the left, so the right is the previous seat; the same direction as
    /// CR 803.1b attack-right). In a duel it is the opponent; alone, nobody.
    PlayerToYourRight,
    /// CR 701.38 — each opponent whose vote in the most recent ballot differed
    /// from the effect's controller's (Grudge Keeper).
    OpponentsWhoVotedDifferently,
    /// Each opponent who voted, on the most recent ballot, for a choice the
    /// controller also voted for (Erestor of the Council, Model of Unity).
    OpponentsWhoVotedTheSame,
    /// A printed "choose an opponent": ONE opponent picked by the controller, not
    /// a fan-out. `GameState::default_hostile_opponent` ranks the choice at 3+
    /// seats; in a duel it is the only opponent. The Rack, Booby Trap.
    HostileOpponent,
    /// "A player of your choice" — not a target (CR 115.10), so a mana
    /// ability naming it stays one (CR 605.1a). `Effect::AddMana` asks it as a
    /// seat ballot led by you; any other reader takes you (Victory Chimes).
    PlayerOfYourChoice,
    /// [`Self::EachWithFewestControlled`]'s mirror: every living player tied
    /// for the most matching permanents, APNAP ("each player who controls the
    /// most creatures investigates" — No Witnesses).
    EachWithMostControlled(Box<SelectionRequirement>),
}

impl PlayerRef {
    /// True for refs naming a *set* of seats (those with a fan-out arm in
    /// `GameState::resolve_players_unranged`).
    ///
    /// `resolve_player` answers these with the set's first seat, which drops the
    /// rest when the set has more than one (`EachPlayer` in a duel hits seat 0
    /// only). An arm that resolves `who` singularly must be handed a ref that is
    /// single here; `resolve_player`'s `debug_assert!` enforces it.
    pub fn is_fan_out(&self) -> bool {
        matches!(
            self,
            PlayerRef::EachOpponent
                | PlayerRef::EachOpponentExceptTriggerer
                | PlayerRef::OpponentsWhoVotedDifferently
                | PlayerRef::OpponentsWhoVotedTheSame
                | PlayerRef::EachPlayer
                | PlayerRef::EachPlayerWithoutMaxSpeed
                | PlayerRef::EachTeammate
                | PlayerRef::EachPlayerExceptControllerOf(_)
                | PlayerRef::EachPlayerSourceAttackedThisTurn
                | PlayerRef::EachPlayerYouAreAttacking
                | PlayerRef::EachWithFewestControlled(_)
                | PlayerRef::EachWithMostControlled(_)
        )
    }
}

/// Which players a player-targeted static affects, read off the source
/// permanent's controller at recompute time (`StaticEffect::PlayerCannotGainLife`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerStaticTarget {
    /// The source's controller — Sulfuric Vortex's "no player can gain
    /// life" applied to the controller-only side. Rare.
    Controller,
    /// Each opponent of the source's controller — "your opponents can't gain
    /// life" (Erebos, God of the Dead).
    ///
    /// ⚠ Not a Tainted Remedy approximation: that card turns gain into *loss*,
    /// which is `StaticEffect::LifeGainBecomesLoss`.
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

/// How a card played off an exile grant may be paid for (CR 609.4b).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExiledPlaySpend {
    /// "Mana of any type can be spent" — the mana value as generic.
    #[default]
    AnyType,
    /// "As though it were mana of any color" — coloured pips become generic.
    AnyColor,
    /// Its own cost.
    Own,
}

// ── Selector ─────────────────────────────────────────────────────────────────

/// A lazy reference to a (possibly empty or multi-) set of permanents, cards
/// in other zones, or players, resolved at execution time; the operand of
/// most [`Effect`] mutations.
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
    /// "Them" in an `Effect::OverTriggerBatch` body: the batch the
    /// dispatcher bound, wherever each card is now (a milled or discarded card
    /// is a graveyard card, not a permanent — `AllTargets` reads only the
    /// battlefield).
    TriggerBatch,
    /// The batch an enclosing [`Effect::WithTriggerBatch`] bound, wherever
    /// each card is now; empty outside one.
    BoundTriggerBatch,
    /// Every declared target slot still legal — "then double the counters on
    /// each of those creatures" (Biogenic Upgrade).
    AllTargets,
    /// [`Selector::AllTargets`] from slot `n` on — "each of X target
    /// creatures" after a first target (Spinning Wheel Kick's damage dealer).
    TargetsFrom(u8),
    /// "…a card at random from among them" — resolve the inner selector and
    /// keep one entity chosen uniformly at random (Volatile Chimera).
    RandomOf(Box<Selector>),
    /// The object that caused this trigger (attacker, dying creature, etc).
    TriggerSource,
    /// The permanent the source chose as it entered
    /// (`CardInstance.chosen_permanent` — Dauntless Bodyguard); nothing once it
    /// has left the battlefield.
    ChosenPermanentOfSource,
    /// The permanent tapped for the source spell's `TapPermanents` cost
    /// (`CardData::cost_tapped` — Swallow Whole); nothing once it has left.
    CostTappedPermanentOfSource,
    /// CR 509.1 — every attacker the source is blocking
    /// (`GameState.block_map[source]`). Wall of Frost.
    BlockedAttacker,
    /// CR 506.3/508.1b — the player, planeswalker or battle the source
    /// creature is attacking (Mage Slayer's "the player or planeswalker it's
    /// attacking"). Empty when the source isn't attacking.
    AttackedBySource,
    /// `AttackedBySource` for the trigger's subject — "the player or
    /// planeswalker that creature is attacking" off an attack trigger watching
    /// other creatures (Raid Bombardment).
    AttackedByTriggerSource,
    /// Mirror of `BlockedAttacker`: every creature blocking the source
    /// (reverse `block_map` lookup). Grasping Giant.
    BlockingCreatures,
    /// The creatures blocking the trigger's subject (She-Hulk: "for each
    /// creature blocking it", "it" being the blocked Hero).
    CreaturesBlockingTriggerSource,
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
    /// the game-level `blocks_declared_this_turn` pair log — so it still resolves after
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
    /// The permanent the current activation exiled to pay its
    /// `exile_permanent_cost` (Curie, Emergent Intelligence's "becomes a copy
    /// of the exiled creature").
    ExiledForCost,
    /// CR 702.95 — the source's soulbond partner (`CardInstance.soulbond_partner`)
    /// while both are on the battlefield; empty when unpaired. Mirage Phalanx.
    SoulbondPartner,

    /// The last card the effect's controller drew this turn, if it's still in
    /// their hand (`Player.last_drawn_card`). Sindbad's "discard it if it isn't
    /// a land", Jandor's Ring's discard cost.
    LastCardYouDrew,

    /// The latest token `Effect::CreateToken` made in this resolution ("create a
    /// token, then put X counters on it" — Fractal Anomaly). Reset per resolution
    /// root.
    LastCreatedToken,
    /// CR 701.47c — "the amassed Army": the Army the current resolution's
    /// last `Effect::Amass` chose, whether or not it got counters (Goblin
    /// Plate Mail).
    AmassedArmy,
    /// The card `Effect::RevealRandomFromHand` last revealed this resolution
    /// (Planeswalker's Mischief exiles it). Empty when nothing was revealed.
    LastRevealedCard,

    /// Every token `Effect::CreateToken` made in this resolution (Fractal
    /// Spawning's "put a +1/+1 counter on each of them"). Reset per resolution
    /// root.
    LastCreatedTokens,
    /// The permanents a proliferate in this resolution put a counter on
    /// (CR 701.34a — Ripples of Potential's "permanents you control that had
    /// a counter put on them this way"). Resets between resolution roots.
    ProliferatedThisResolution,
    /// CR 702.51 — the creatures that convoked the effect's source as it was
    /// cast this turn (Venerated Loxodon).
    CreaturesThatConvokedSource,
    /// Each player who controls a permanent matching the filter (Bonder's
    /// Ornament).
    PlayersControlling(SelectionRequirement),
    /// The controller's `count` greatest-power permanents matching `filter`
    /// — the engine's pick for "up to X target creatures" (Vitality Hunter).
    GreatestPowerTopN { filter: SelectionRequirement, count: Value },
    /// "Any number of [filter] you control with different names" — one
    /// permanent per name, the first in battlefield order (Battle for
    /// Bretagard's chapter III).
    OnePerDistinctName(SelectionRequirement),

    /// Every card moved by `Effect::Move` (and Mill / Exile shortcuts) this
    /// resolution, so a following `GrantMayPlay` hits the same cards (Practiced
    /// Scrollsmith). Reset per resolution root.
    LastMoved,
    /// The last source that damaged the inner permanent this turn
    /// (`CardInstance.damaged_by_this_turn`) — Vraska the Unseen.
    LastDamagerOf(Box<Selector>),
    /// Every creature in the sector picked by the enclosing
    /// `Effect::ChooseSector`.
    CreaturesInChosenSector,
    /// The permanent sacrificed to pay this spell's / ability's cost
    /// (`GameState.sacrificed_card`, stamped by `Effect::WithSacrificedPt`), now
    /// a graveyard card. Rescue from the Underworld.
    SacrificedCard,
    /// The card(s) exiled from a graveyard to pay this activation's cost
    /// (Necropolis). Empty outside such an activation.
    CostExiledCards,
    /// The cards sacrificed during this resolution that are now in a
    /// graveyard and match `filter` — Danse Macabre's "creature cards put
    /// into a graveyard this way".
    SacrificedThisResolution { filter: SelectionRequirement },
    /// Every player and planeswalker the source has dealt damage to this
    /// game (The Fallen).
    DamagedBySourceThisGame,
    /// Each opponent dealt combat damage this game by a creature with the
    /// source's name — any such creature, not only this object (Gollum,
    /// Obsessed Stalker).
    OpponentsCombatDamagedByOwnName,

    /// Target slot N (0-indexed) of the spell whose cast fired this trigger, read
    /// off the topmost matching `StackItem::Spell`; empty when the source isn't a
    /// spell or the slot is unfilled. Repartee payoffs that act on the spell's
    /// target (Conciliator's Duelist).
    CastSpellTarget(u8),
    /// Every permanent the just-cast spell targets, across all its slots
    /// ("gain control of those permanents" — Dack Fayden's emblem).
    AllCastSpellTargets,

    /// CR 702 Radiance — `subject` (usually a target slot) plus every other
    /// permanent sharing a card type and a computed color with it; a colorless
    /// subject is just itself. The non-damage sibling of `Effect::RadianceDamage`
    /// (Rally the Righteous, Leave No Trace).
    RadianceGroup { subject: Box<Selector> },

    /// All game objects matching `filter` in `zone`.
    EachMatching { zone: ZoneRef, filter: SelectionRequirement },
    /// All permanents on the battlefield matching `filter`.
    EachPermanent(SelectionRequirement),
    /// All battlefield permanents matching `filter` EXCEPT the cast-time
    /// target list — the "choose one, [destroy] the rest" shape (Deadly
    /// Vanity: "Choose a creature or planeswalker. Destroy the rest.").
    EachPermanentExceptTargets(SelectionRequirement),
    /// All battlefield permanents matching `filter` EXCEPT the tokens this
    /// resolution just created (`LastCreatedTokens`) — "create X Soldiers …
    /// destroy all other creatures" (Martial Coup).
    EachPermanentExceptLastCreated(SelectionRequirement),
    /// `who`'s `filter` permanents — player-relative `EachPermanent` for a
    /// targeted player (Sleep).
    ControlledBy { who: PlayerRef, filter: SelectionRequirement },
    /// All battlefield permanents *owned* by `who` matching `filter` — the
    /// ownership twin of `ControlledBy` ("a permanent owned by the voter" —
    /// Expropriate), so a stolen permanent still answers to its owner.
    OwnedBy { who: PlayerRef, filter: SelectionRequirement },
    /// Every creature controlled by the controller of `subject`, **except**
    /// `subject` itself ("other creatures that player controls" — Mark for
    /// Death). Empty if `subject` resolves to nothing.
    OtherCreaturesControlledByControllerOf(Box<Selector>),
    /// The controller's creature with the least toughness (first in battlefield
    /// order on a tie); empty with none. Bolster (CR 701.21).
    LeastToughnessYouControl,
    /// The controller's creature with the greatest power (first on a tie); empty
    /// with none. Triumph of Gerrard (modeled non-targeted).
    GreatestPowerYouControl,
    /// The single permanent `who` controls with the greatest mana value among
    /// those matching `filter` (Juxtapose). Ties go to the first in
    /// battlefield order, standing in for the printed controller's choice.
    GreatestManaValueControlledMatching { who: PlayerRef, filter: SelectionRequirement },
    /// The controller's creature matching `filter` with the greatest power. Empty
    /// reads as 0 through `Value::PowerOf`, so `Max(Const(n), PowerOf(..))` floors
    /// at n (Triumphant Chomp).
    GreatestPowerControlledMatching(SelectionRequirement),
    /// The controller's *other* creature (excluding the source) with the greatest
    /// toughness, first on a tie; empty with none. Flourishing Hunter.
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
    /// CR 702.22 — the other creatures in the attacking band `anchor`
    /// belongs to this combat ("all creatures banded with it" — Icatian
    /// Skirmishers, Urza's Engine). Empty outside a declared band.
    BandedWith(Box<Selector>),
    /// CR 702.6e — the Aura/Equipment granting the resolving ability, for granted
    /// lines that name their granter (Hankyu). Two granters on one host are
    /// indistinguishable, so all of them match.
    AttachmentGranting,
    /// A `filter` card from the controller's hand ("reveal a [card] in your
    /// hand" — Assembly Hall): the first match in hand order, since selectors
    /// have no decision hook. Nothing on no match.
    ChosenCardInHand(SelectionRequirement),
    /// One `filter` permanent or player chosen uniformly at random (Goblin Test
    /// Pilot); never a target slot. Empty pool, nothing.
    RandomAmong(SelectionRequirement),

    /// Top `count` cards of `who`'s library.
    TopOfLibrary { who: PlayerRef, count: Value },
    /// The top cards of `who`'s library, walked down until their mana values sum
    /// to at least `threshold` (the card that crosses it is included); the whole
    /// library if it never does. Improvisation Capstone's "until you exile cards
    /// with total mana value 4 or greater".
    TopOfLibraryUntilMvAtLeast { who: PlayerRef, threshold: Value },
    /// Bottom `count` cards of `who`'s library.
    BottomOfLibrary { who: PlayerRef, count: Value },
    /// Every card in `who`'s zone matching `filter`.
    CardsInZone { who: PlayerRef, zone: Zone, filter: SelectionRequirement },

    /// Cards (any player's) discarded earlier in this resolution matching
    /// `filter`, found in the graveyard via
    /// `GameState.scratch.discarded_card_ids_this_resolution`. Mind Roots.
    DiscardedThisResolution { filter: SelectionRequirement },

    /// Cards exiled earlier in this resolution matching `filter`
    /// (`scratch.exiled_card_ids_this_resolution`). Bonehoard Dracosaur.
    ExiledThisResolution { filter: SelectionRequirement },

    /// Entities damaged earlier in this resolution: players always, permanents
    /// matching `filter` (`GameState.scratch.damaged_this_resolution`). Aurelia's
    /// Fury.
    DamagedThisResolution { filter: SelectionRequirement },

    /// Cards destroyed earlier in this resolution matching `filter`, wherever
    /// they are now (`GameState.scratch.destroyed_this_resolution`). Cleansing
    /// Meditation.
    DestroyedThisResolution { filter: SelectionRequirement },

    /// A single player, lifted to selector form.
    Player(PlayerRef),

    /// Narrow another selector's result set to the entities matching `filter`
    /// ("the *nonblack* creature blocking or blocked by this" — Deathgazer).
    /// Player entities never match a permanent filter and are dropped.
    MatchingAmong { inner: Box<Selector>, filter: SelectionRequirement },
    /// CR 701.54 — `who`'s Ring-bearer, if they have one on the battlefield
    /// (Galadriel, Elven-Queen's "put a +1/+1 counter on your Ring-bearer").
    RingBearerOf(PlayerRef),
    /// At most `count` entities of `inner`, in resolution order ("up to two
    /// creature cards from your graveyard" — Pull from the Grave). `count` is
    /// evaluated in the controller's resolution context.
    Take { inner: Box<Selector>, count: Box<Value> },
    /// Like `Take` but picks `count` entities uniformly at random instead of
    /// the first in resolution order (Capricious Hellraiser's random exile).
    TakeRandom { inner: Box<Selector>, count: Box<Value> },
    /// Like `Take` but keeps the `count` entities with the greatest power
    /// (ties keep resolution order) — a permanent's current power, a card's
    /// printed one: Highcliff Felidar, Stitcher Geralf's exiled cards.
    TakeGreatestPower { inner: Box<Selector>, count: Box<Value> },

    /// Every battlefield permanent (the anchor included) with the same printed
    /// name as `inner` (usually `Target(0)`); empty if `inner` is not a permanent.
    /// Maelstrom Pulse, Echoing Truth.
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
    /// The first selector's entities minus the second's — "all *other* …"
    /// after a sacrifice (The Promise of Kahless: the graveyard's Klingons
    /// except the one just sacrificed).
    Except(Box<Selector>, Box<Selector>),
    /// The permanents this source assimilated (`Effect::Assimilate`) — The
    /// Best of Both Worlds' "the Borg assimilated with this Saga".
    AssimilatedBySource,

    /// One pile of the enclosing `Effect::SeparateIntoPiles`: `chosen` or the
    /// leftover (`GameState.separated_piles`); battlefield ids resolve as
    /// permanents, the rest as cards.
    SeparatedPile { chosen: bool },

    /// Exactly these objects, in order; battlefield ids resolve as permanents,
    /// the rest as cards.
    ///
    /// Runtime-only: `Effect::ForEach`'s suspend splice builds it so unreached
    /// entities survive the suspension. No card names a `CardId`;
    /// `scripts/audit_variant_coverage.py` lists it as dead by design.
    ExactObjects(Vec<crate::card::CardId>),
    /// `inner`'s permanents with power greater than `than`, read once before any
    /// is affected (Fell the Mighty). Cards elsewhere compare printed power.
    PowerAbove { inner: Box<Selector>, than: Box<Value> },

    /// No entities (placeholder/default).
    None,
    /// `TakeGreatestPower` by mana value instead (ties in resolution order) — the
    /// auto-pick for "choose a card exiled this way; you may cast it" (Author of
    /// Shadows).
    TakeGreatestManaValue { inner: Box<Selector>, count: Box<Value> },
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

    /// `Selector::TakeGreatestManaValue { inner, count }` — the engine's
    /// pick for a choice the controller would spend on their best card.
    pub fn take_priciest(inner: Selector, count: Value) -> Self {
        Selector::TakeGreatestManaValue { inner: Box::new(inner), count: Box::new(count) }
    }

    /// The priciest one of `inner` (`take_priciest` with a count of one).
    pub fn best_of(inner: Selector) -> Self {
        Selector::take_priciest(inner, Value::Const(1))
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
    /// Entities of `sel` also matching `filter` — a filtered count of a non-zone
    /// selector like `LastMoved` (Culmination of Studies).
    CountMatching { sel: Box<Selector>, filter: SelectionRequirement },
    /// Sum of the mana values of the entities the selector resolves to
    /// (Summon: Bahamut's IV — "total mana value of other permanents you
    /// control"). Off-battlefield objects contribute their printed CMC.
    TotalManaValueOf(Box<Selector>),
    PowerOf(Box<Selector>),
    /// The printed (copiable) power of the first entity `what` resolves to —
    /// "draw cards equal to its base power" (Curie, Emergent Intelligence).
    BasePowerOf(Box<Selector>),
    /// CR 700.8 — party size: distinct roles (Cleric, Rogue, Warrior, Wizard)
    /// among the controller's creatures, one role per creature, max 4.
    PartyCount,
    /// Number of creatures currently blocking the resolved permanent
    /// (Spined Sliver's "+1/+1 for each creature blocking it").
    BlockersOf(Box<Selector>),
    /// CR 509.1b — number of attackers the resolved permanent is blocking
    /// (Guardian of the Gateless's "+1/+1 for each creature it's blocking").
    CreaturesBlockedBy(Box<Selector>),
    ToughnessOf(Box<Selector>),
    /// The printed toughness of the selected permanents — no layer effects,
    /// so a static that scales with its own host's toughness can read it
    /// without reading itself (Snowblind's "toughness minus 1").
    BaseToughnessOf(Box<Selector>),
    /// Marked damage on the selector's first entity (CR 120.3), from the live
    /// permanent or its CR 603.10 LKI, so a dies-trigger can read "damage dealt
    /// to it this turn" (Tangled Colony). Damage removed mid-turn is missed.
    MarkedDamageOn(Box<Selector>),
    /// CR 120.10 — the excess damage dealt this turn to the selected
    /// creatures (`excess_damage_this_turn`), read off LKI once one has died:
    /// "where X is that excess damage" (Overclocked Electromancer).
    ExcessDamageDealtThisTurnTo(Box<Selector>),
    LifeOf(PlayerRef),
    /// CR 702.179 — `who`'s current speed (0–4). Momentum Breaker's "gain
    /// life equal to your speed". Backed by `Player.speed`.
    PlayerSpeed(PlayerRef),
    /// Lowest life total among all players in the game (CR 119.7 — Repay
    /// in Kind: "each player's life total becomes the lowest life total
    /// among all players").
    LowestLifeTotal,
    /// `who`'s life total as the current turn began (`Player.life_at_turn_start`)
    /// — Sengir, the Dark Baron's "that player's life total as the turn
    /// began".
    LifeAtTurnStart(PlayerRef),
    /// The lowest life total among the controller's opponents still in the
    /// game; 0 with none. Breena's "more life than another of your opponents".
    LowestOpponentLife,
    /// The most permanents matching the filter any one opponent of the
    /// controller controls; 0 with none. Boreas Charger's "an opponent who
    /// controls more lands than you".
    MostControlledByAnOpponent(crate::card::SelectionRequirement),
    /// The highest life total among all (alive) players. Sorin, Grim
    /// Nemesis −9.
    HighestLifeTotal,
    HandSizeOf(PlayerRef),
    /// The unspent mana in a player's pool (Ozai, the Phoenix King's "as long
    /// as you have six or more unspent mana"); the largest for a multi-seat
    /// ref.
    UnspentManaOf(PlayerRef),
    /// The number of the source controller's opponents who have `n` or fewer
    /// cards in hand. Powers "draw an additional card for each opponent who
    /// has one or fewer cards in hand" (Bandit's Talent, level 3).
    OpponentsWithHandSizeAtMost(u32),
    /// The number of the source controller's opponents whose life total is
    /// less than half their starting life total (Anya, Merciless Angel;
    /// CR 103.4 — 20 in a duel, 40 in Commander).
    OpponentsBelowHalfStartingLife,
    /// How many times the source's activated abilities were activated this
    /// turn — the ability must record its uses (`max_activations_per_turn`;
    /// Dragon Whelp's uncapped `Some(u32::MAX)`).
    SourceActivationsThisTurn,
    /// Opponents of the controller holding at least N cards (Wolfcaller's
    /// Howl).
    OpponentsWithHandSizeAtLeast(u32),
    /// Living players whose `tally` is strictly greater than the source
    /// controller's (Oreskos Explorer: "the number of players who control
    /// more lands than you").
    PlayersWithGreaterTally(crate::card::PlayerTally),
    /// Living players with a strictly smaller `tally` than the source's
    /// controller, never the controller (Birth of the Imperium). Mirror of
    /// `PlayersWithGreaterTally`.
    PlayersWithFewerTally(crate::card::PlayerTally),
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
    /// Life `who` has gained this turn (CR 119.3; `Player.life_gained_this_turn`).
    /// Accomplished Alchemist.
    LifeGainedThisTurn(PlayerRef),
    /// Life lost this turn (CR 119.3), max over seats, so `EachOpponent` reads the
    /// most any opponent lost (Spinerock Knoll). `Player.life_lost_this_turn`.
    LifeLostThisTurn(PlayerRef),
    /// The *sum* of the life the players `who` resolves to lost this turn —
    /// "the total amount of life your opponents lost this turn" (Florian,
    /// Voldaren Scion). The summing sibling of `LifeLostThisTurn`'s max.
    TotalLifeLostThisTurn(PlayerRef),
    /// +1/+1 counters the player put on creatures they control this turn
    /// (Iridescent Hornbeetle). Read from `GameState::own_p1p1_counters`.
    PlusOneCountersPutOnYourCreaturesThisTurn(PlayerRef),
    /// The {E} the players `who` resolves to have (summed) — Razorfield
    /// Ripper's "+X/+X, where X is the amount of {E} you have".
    EnergyOf(PlayerRef),
    /// The {E} the players `who` resolves to paid or lost this turn (summed;
    /// `Player.energy_spent_this_turn`) — Blaster Hulk's cost reduction.
    EnergySpentThisTurn(PlayerRef),
    /// Damage dealt to a player this turn (`Player.damage_taken_this_turn`) —
    /// "Bloodthirst X, where X is the damage dealt to your opponents this
    /// turn" (Petrified Wood-Kin). Sums across the resolved players.
    DamageTakenThisTurn(PlayerRef),
    /// The most damage any one of the resolved players was dealt this turn
    /// (`Player.damage_taken_this_turn`, MAX not sum) — "if an opponent was
    /// dealt 3 or more damage this turn" (Lightning Phoenix).
    MostDamageTakenThisTurn(PlayerRef),
    /// CR 706 — dice the resolved players rolled this turn, summed
    /// ("as long as you've rolled three or more dice this turn" — The Space
    /// Family Goblinson; "your third die each turn" — Resolute Veggiesaur).
    DiceRolledThisTurn(PlayerRef),
    /// CR 718.5 — Attractions the resolved players visited this turn, summed
    /// ("for each Attraction you've visited this turn" — Squirrel Squatters).
    AttractionsVisitedThisTurn(PlayerRef),
    /// CR 303.4 — how many `filter` permanents are attached to the players
    /// `who` resolves to ("the number of Curses attached to that player" —
    /// Curse of Thirst; "each Aura attached to you").
    AttachedToPlayerMatching { who: PlayerRef, filter: crate::card::SelectionRequirement },
    /// The noncombat slice of `DamageTakenThisTurn`, summed across the
    /// resolved players (Chandra's Incinerator's cost reduction).
    NoncombatDamageTakenThisTurn(PlayerRef),
    /// Noncombat damage red sources the resolved players controlled dealt
    /// this turn, to anything (Temple of Power's activation gate).
    RedNoncombatDamageDealtThisTurn(PlayerRef),
    /// Combat-only slice of `DamageTakenThisTurn`, max over seats
    /// (`Player.combat_damage_taken_this_turn`). Sidequest: Play Blitzball.
    CombatDamageTakenThisTurn(PlayerRef),
    /// How many of the resolved players were dealt combat damage this turn
    /// (`combat_damage_taken_this_turn > 0`) — "the number of opponents that
    /// were dealt combat damage this turn" (Tymna the Weaver).
    PlayersDealtCombatDamageThisTurn(PlayerRef),
    /// [`Value::PlayersDealtCombatDamageThisTurn`] narrowed to damage from a `by`
    /// creature (read with the source as "this"). Estinien Varlineau.
    PlayersDealtCombatDamageThisTurnBy { who: PlayerRef, by: Box<crate::card::SelectionRequirement> },
    /// How many players control at least `n` more lands than you (Surveyor's
    /// Scope's "players who control at least two more lands than you").
    PlayersWithLandsAtLeastMore(u32),
    /// Damage dealt to the source this turn
    /// (`CardInstance.damage_dealt_to_this_turn`, LKI for dies-triggers).
    /// Rushing-Tide Zubera.
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
    /// `filter` cards `who` owns in exile (Candlekeep Inspiration); add a
    /// graveyard count for "in exile and in your graveyard".
    CardsInExileOwnedMatching { who: PlayerRef, filter: SelectionRequirement },
    /// Distinct card types among cards in `who`'s graveyard (the delirium
    /// count as a number — Lucid Dreams' "draw X").
    CardTypesInGraveyard(PlayerRef),
    /// Supertypes + card types + subtypes the first entity `what` names has,
    /// read off its computed type line (Embiggen).
    TypeLineCountOf(Box<Selector>),
    /// Distinct *permanent* types (artifact, battle, creature, enchantment,
    /// land, planeswalker — CR 110.4a) among cards in `who`'s graveyard
    /// (Matzalantli, the Great Door's transform gate).
    PermanentTypesInGraveyard(PlayerRef),
    /// Distinct card types among permanents `who` controls and cards in
    /// their graveyard, together (Happily Ever After's "six or more card
    /// types among permanents you control and/or cards in your graveyard").
    CardTypesAmongPermanentsAndGraveyard(PlayerRef),
    /// Distinct card types among the cards `sel` names, in whatever zone
    /// (Occult Epiphany's "for each card type among cards discarded this
    /// way", over `Selector::DiscardedThisResolution`).
    CardTypesAmong(Box<Selector>),
    /// Opponents of the controller who control at least one permanent `sel`
    /// names (Sudden Salvation's "for each opponent who controls one or more
    /// of those permanents").
    OpponentsControllingAnyOf(Box<Selector>),
    /// The just-cast spell's distinct targets matching the filter (a
    /// `SpellCast` trigger) — "put that many +1/+1 counters" (Arcee).
    CastSpellTargetsMatching(crate::card::SelectionRequirement),
    /// Cards `who` has discarded this turn (max over resolved players).
    /// Backed by `Player.cards_discarded_this_turn` (Dihada's Ploy).
    CardsDiscardedThisTurn(PlayerRef),
    /// Cards put into `who`'s graveyard from anywhere this turn (max over seats;
    /// `Player.cards_to_graveyard_this_turn`, the CR 700.4 tally). Fraying Sanity.
    CardsPutIntoGraveyardThisTurn(PlayerRef),
    /// Cards put into `who`'s graveyard this turn from anywhere other than the
    /// battlefield (Dimir Strandcatcher): the turn's graveyard arrivals minus
    /// the ones `deaths.graveyard_from_battlefield_this_turn` recorded.
    CardsPutIntoGraveyardNotFromBattlefieldThisTurn(PlayerRef),
    /// Creature cards put into `who`'s graveyard from anywhere this turn
    /// (`Player.creature_cards_to_graveyard_this_turn`) — Raphael, Fiendish
    /// Savior's end-step gate.
    CreatureCardsPutIntoGraveyardThisTurn(PlayerRef),
    /// Distinct mana costs among nonland cards in `who`'s graveyard (max over
    /// seats). {2}{B} ≠ {1}{B}{B}; no mana cost isn't counted. Embodiment of
    /// Agonies.
    DistinctManaCostsInGraveyard(PlayerRef),
    /// Cards put into `who`'s graveyard from their hand (discarded) or
    /// library (milled) this turn — Welcome the Dead's X. Surveil isn't
    /// counted.
    CardsPutIntoGraveyardFromHandOrLibraryThisTurn(PlayerRef),
    /// Number of card types on the most recently discarded card (Mount
    /// Velus Manticore's "X = the number of card types the discarded card
    /// has"). Backed by `GameState.last_discarded_card_types`.
    LastDiscardedCardTypes,
    /// Mana value of the most recently discarded card, falling back to the one
    /// discarded to pay the resolving ability's cost (Slumbering Tora's X).
    /// 0 when nothing has been discarded.
    LastDiscardedManaValue,
    /// CR 601.2h — the total mana value of the cards discarded to pay the
    /// resolving spell's cost, stamped on it at cast (`cast_discarded_mana_value`;
    /// copies carry it, CR 707.10). Nahiri's Wrath.
    CastDiscardedManaValue,
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
    /// Noncreature spells `who` has cast this turn, max over seats, including the
    /// triggering one (`Player.noncreature_spells_cast_this_game_turn`). Magebane
    /// Lizard.
    NoncreatureSpellsCastThisTurn(PlayerRef),
    /// Total spells `who` has cast so far this turn (max over resolved
    /// players). Backed by `Player.spells_cast_this_game_turn` — Narset, Jeskai
    /// Waymaster draws this many after discarding her hand.
    SpellsCastThisTurn(PlayerRef),
    /// CR 700.6 — historic spells `who` has cast this turn
    /// (`Player.historic_spells_cast_this_turn`): Peri Brown's "the first
    /// historic spell you cast each turn".
    HistoricSpellsCastThisTurn(PlayerRef),
    /// Total spells cast this turn across every player (Erayo). The summed
    /// sibling of `SpellsCastThisTurn`, which takes the max over seats.
    SpellsCastThisTurnTotal,
    /// Spells and abilities that caused a player to guess or to group cards
    /// or permanents into a pile this turn (The Celestial Toymaker).
    PileOrGuessEffectsThisTurn,
    /// The greatest mana value the source permanent itself noted this turn as
    /// cards were put into exile while it was on the battlefield (Bell Borca,
    /// Spectral Sergeant: "noted **for it**"). Backed by `CardCold::exile_note`.
    GreatestManaValueNotedForSource,
    /// Spells `who` has cast this turn other than the source itself (the count
    /// minus one, floored at 0): Thunder Salvo's "other spells you've cast".
    OtherSpellsCastThisTurn(PlayerRef),
    /// Instant and sorcery spells `who` has cast this turn, summed over seats
    /// (`Player.instants_or_sorceries_cast_this_turn`). Rionya, Fire Dancer.
    InstantsOrSorceriesCastThisTurn(PlayerRef),
    /// Total mana value of `who`'s spells cast this turn other than the source,
    /// summed over seats. A spell that has left the stack reads its printed mana
    /// value (X = 0). Call Forth the Tempest.
    TotalManaValueOfOtherSpellsCastThisTurn(PlayerRef),
    /// Greatest mana value among `who`'s instants and sorceries cast this turn,
    /// stamped at cast so X counts (CR 202.3e). Rootha, Mastering the Moment.
    GreatestInstantOrSorceryManaValueCastThisTurn(PlayerRef),
    /// Creatures `who` declared as attackers this turn (max over seats); ones put
    /// onto the battlefield attacking don't count (Windbrisk Heights ruling).
    CreaturesAttackedWithThisTurn(PlayerRef),
    /// CR 702.121 Melee — distinct opponents the active player attacked this
    /// combat, read from `GameState.attacking` (planeswalkers and battles map to
    /// their defending player).
    OpponentsAttackedThisCombat,
    /// The number of distinct opponents the context's controller attacked
    /// (with a creature attacking that player, CR 508.1) this turn — Fast
    /// Forward's "for each opponent you attacked this turn".
    OpponentsAttackedThisTurn,
    /// Creatures attacking the player `PlayerRef` resolves to — "the number
    /// of creatures attacking them" (Within Range), planeswalkers and battles
    /// not counted (CR 506.3).
    CreaturesAttackingPlayer(PlayerRef),
    /// Creatures attacking the player `who` resolves to that match `filter`
    /// for the context's controller — "one or more Dragons you control attack
    /// that player" (Firkraag).
    AttackersOfPlayerMatching { who: PlayerRef, filter: SelectionRequirement },
    /// The game's current turn number (CR 500 — the first turn is 1). Powers
    /// "the first upkeep" gates (Sentinel Dispatch).
    TurnNumber,
    /// Equip abilities `who` activated this turn (Forge Anew's first equip).
    EquipsActivatedThisTurn(PlayerRef),
    /// Turns `who` has begun this game, the current one included (Serra Avenger:
    /// `<= 3`). Extra turns count; skipped ones never begin (CR 500.7, 500.11).
    TurnsBegunBy(PlayerRef),
    /// CR 905.2b — a number the controller noted as they drafted cards with
    /// this source's name. `Max` takes the highest note (Lurking Automaton),
    /// `Sum` totals them (Cogwork Grinder). Zero outside a drafted game.
    DraftNoteNumber { agg: DraftNoteAgg },
    GraveyardSizeOf(PlayerRef),
    /// The largest graveyard among `who`'s players — "an opponent has N or more
    /// cards in their graveyard" (Jace's Phantasm). `GraveyardSizeOf` reads only
    /// a fan-out's first seat.
    GreatestGraveyardSizeAmong(PlayerRef),
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
    /// `filter` cards in `who`'s hand — Amplify (CR 702.38; all matches
    /// auto-revealed).
    CardsInHandMatching { who: PlayerRef, filter: SelectionRequirement },
    /// Largest graveyard among living players ("if a graveyard has 20 or more
    /// cards" — Visions of Beyond). Compare `GraveyardSizeOf`.
    MaxGraveyardSize,
    /// How many alive players' graveyards hold at least `n` cards — "draw a
    /// card for each graveyard with seven or more cards in it" (The Master of
    /// Lake-town).
    GraveyardsWithAtLeast(u32),
    /// Greatest printed power among the selector's cards in any zone; 0 when
    /// none. Szat's Will.
    GreatestPowerAmongCards(Box<Selector>),
    /// The greatest computed power *or* toughness among the battlefield
    /// permanents the selector resolves to; 0 when none (Freelance Muscle's
    /// "greatest power and/or toughness among other creatures you control").
    GreatestPowerOrToughnessAmong(Box<Selector>),
    /// Number of cards in `who`'s library. Used by Body of Research's
    /// "for each card in your library" Fractal-token scaling.
    LibrarySizeOf(PlayerRef),
    /// The X value paid in the spell's cost.
    XFromCost,
    /// 1 while `pred` holds, else 0 — a predicate as a number ("if this
    /// spell was kicked, any number instead": The Eagles Are Coming!'s
    /// target cap).
    OneIf(Box<Predicate>),
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
    /// CR 700.5 devotion: mana symbols of the listed colors among the controller's
    /// permanents' costs; hybrid / Phyrexian pips count for each color half. Gray
    /// Merchant of Asphodel.
    DevotionTo(Vec<crate::mana::Color>),
    /// The greatest number of the controller's creatures that share a
    /// creature type (Skemfar Shadowsage — "creatures you control that have
    /// a creature type in common"). Changelings count for every type.
    LargestCreatureTypeCount,
    /// The number of different creature types among creatures the controller
    /// controls (Valiant Changeling's discount). A changeling has every type,
    /// so it saturates the count.
    DistinctCreatureTypesAmongYourCreatures,
    /// The Federation count — creature types among the controller's non-Borg
    /// creatures (Hoshi Sato, Exolinguist).
    DistinctCreatureTypesAmongYourNonBorgCreatures,
    /// The Federation count over the non-Borg creatures you ALREADY control —
    /// the context's trigger source (an enters-with rider's entrant, CR
    /// 614.12) excluded (Jean-Luc, Mon Capitaine).
    DistinctCreatureTypesAmongYourOtherNonBorgCreatures,
    /// The greatest number of cards any one opponent of the controller has
    /// drawn this turn (Thought Sponge).
    GreatestCardsDrawnThisTurnByAnOpponent,
    /// Counters of the given type on `what`.
    CountersOn { what: Box<Selector>, kind: CounterType },
    /// All counters (every kind) on `what` — "for each counter on it"
    /// (Twitching Doll). Sums every counter the permanent carries.
    TotalCountersOn { what: Box<Selector> },
    /// CR 702.143 — foretold cards `who` owns in exile: face down with a
    /// foretell cost, printed or granted (Niko Defies Destiny's chapter I).
    ForetoldCardsOwnedInExile(PlayerRef),
    /// CR 120.10 — excess damage dealt this resolution
    /// (`GameState.excess_damage_this_resolution`). The Last Agni Kai.
    ExcessDamageDealtThisResolution,
    /// Face-down creatures on the battlefield, any controller (Ixidron's
    /// characteristic-defining power/toughness).
    FaceDownCreatures,
    /// Total damage that actually landed during this resolution — "gain life
    /// equal to the damage dealt this way" (Brightflame).
    DamageDealtThisResolution,
    /// Mana spent on the spell countered this resolution
    /// (`GameState.countered_spell_mana_spent`, from `Effect::CounterSpell`).
    /// Mana Sculpt.
    CounteredSpellManaSpent,
    /// Mana value of the spell countered this resolution, the printed-cost
    /// sibling of `CounteredSpellManaSpent` (Plasm Capture).
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
    /// CR 702.119 — the toughness of the creature sacrificed to cast the
    /// source for its emerge cost this turn; 0 when it wasn't emerged
    /// (Adipose Offspring).
    EmergeSacrificedToughness,
    /// Nonland cards exiled by the enclosing
    /// [`Effect::ExileTopBatchesUntilLandLast`] (Rally the Horde).
    NonlandCardsExiledThisEffect,
    /// The `color` mana spent casting the source permanent, from its stamped
    /// cast provenance (`CardInstance.cast_mana_spent_by_color`) — Clay
    /// Champion's "for each {G}{G} spent to cast it". 0 when it wasn't cast.
    SourceCastManaOfColor(crate::mana::Color),
    /// CR 702.82 — the creatures the source devoured that match the filter,
    /// read as they last existed (Voracious Dragon's Goblins).
    DevouredMatching(crate::card::SelectionRequirement),
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
    /// The power of the card revealed to pay this cast's
    /// `AdditionalCastCost::RevealFromHand` (Titan's Presence). 0 with no reveal.
    RevealedForCostPower,
    /// The mana value of that revealed card (Disaster Radius); the reveal
    /// picks the match with the greatest mana value for it.
    RevealedForCostManaValue,
    /// Power of the creature last sacrificed this resolution (set by
    /// `Effect::SacrificeAndRemember`). Thud.
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
    /// Card types among the permanents this activation's cost sacrificed
    /// (`scratch.cost_sacrificed_batch`, read off the last-known
    /// snapshots) — Baba Lysaga, Night Witch.
    CardTypesAmongCostSacrificed,
    /// Commanders (any player's, CR 903.3) sacrificed during this resolution
    /// (Tevesh Szat's "if the sacrificed permanent was a commander").
    CommandersSacrificedThisResolution,
    /// Permanents `who` sacrificed during this resolution that matched
    /// `filter` as they were sacrificed — "for each land sacrificed this way,
    /// its controller may search …" (Wave of Vitriol).
    SacrificedThisResolutionBy { who: PlayerRef, filter: SelectionRequirement },
    /// CR 702.184a — power of the creature tapped to pay a Station ability's
    /// cost, carried to resolution by `Effect::WithTappedPower`.
    TappedForCostPower,
    /// Mana from Treasures spent to activate this ability ("an additional
    /// six-sided die for each mana from Treasures spent to activate this
    /// ability", Mr. House). Carried by `Effect::WithTreasureManaSpent`.
    TreasureManaSpentToActivate,
    /// Times the latest `Effect::MayPayRepeatedly` was paid this resolution
    /// (Bloodthirsty Adversary).
    TimesPaidThisEffect,
    /// Toughness of the creature last sacrificed this resolution (set by
    /// `Effect::SacrificeAndRemember`). Tribute to Hunger.
    SacrificedToughness,
    /// Mana value of the most recently sacrificed permanent this
    /// resolution (set by `Effect::SacrificeAndRemember`). Reckoner's
    /// Bargain ("gain life equal to the sacrificed permanent's mana value").
    SacrificedManaValue,
    /// Mana value of what this activation's cost exiled (`exile_permanent_cost`
    /// — Food Chain), unattached (`unattach_equipment_cost`) or took off the
    /// stack (`exile_spell_cost` — Synthesis Pod).
    ExiledForCostManaValue,
    /// Number of cards in *every* graveyard sharing the resolving source's
    /// name (Accumulated Knowledge).
    CardsNamedLikeSourceInAllGraveyards,
    /// Number of cards in *every* graveyard sharing the name of the spell that
    /// fired this trigger (the Odyssey Shrine cycle).
    CardsNamedLikeTriggerSpellInAllGraveyards,
    /// Cards discarded so far in this resolution (every `CardDiscarded` from
    /// `Effect::Discard` / `DiscardChosen`), so `Seq([Discard, Draw])` draws what
    /// it discarded (Borrowed Knowledge). Reset per resolution.
    CardsDiscardedThisEffect,
    /// Number of cards drawn so far within the current effect resolution — the
    /// draw-side twin of `CardsDiscardedThisEffect` ("draws that many cards,
    /// then discards that many cards" — Laquatus's Creativity).
    CardsDrawnThisEffect,
    /// Mana value of the last card exiled within the current resolution
    /// (Undying Flames' "damage equal to that card's mana value"). 0 when
    /// nothing was exiled. Reset between independent resolutions.
    LastExiledManaValue,
    /// Power of the last card exiled within the current resolution, as it last
    /// existed on the battlefield (CR 603.10 / 608.2h — Olórin's Searing
    /// Light's "the power of the creature they exiled"). 0 when nothing was.
    LastExiledPower,
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
    /// Cards a surveil earlier in the current resolution left on top of the
    /// library ("for each card you put on top of your library" — Starving
    /// Revenant). Reset between independent resolutions.
    SurveilKeptOnTopThisEffect,
    /// The mana value of the card the last `Effect::Discover` hit this
    /// resolution, or its cap N when it hit nothing (Hit the Mother Lode).
    DiscoveredManaValue,
    /// Parley's other count: the land cards revealed by the enclosing
    /// `Effect::Parley` (Phabine's Citizen tokens).
    LandCardsRevealedThisEffect,
    /// The greatest mana value among the cards the enclosing `Effect::Parley`
    /// revealed (Alexander, Ship's Fool).
    ParleyGreatestManaValue,
    /// {E} paid by an `Effect::PayAnyEnergy` earlier this resolution (Aether
    /// Spike's `extra_generic`).
    EnergyPaidThisEffect,
    /// The most cards any one player discarded in this resolution
    /// (`scratch.cards_discarded_per_player_this_resolution`): Windfall's "draws
    /// cards equal to the greatest number of cards a player discarded this way".
    MaxCardsDiscardedThisEffectByAnyPlayer,
    /// Nonland cards `who` discarded in this resolution, summed
    /// (`scratch.nonland_cards_discarded_per_player_this_resolution`); count
    /// sibling of `Predicate::DiscardedNonlandThisEffect`. Garruk, Veiled Butcher.
    NonlandCardsDiscardedThisEffectBy(PlayerRef),
    /// Creature cards discarded so far in this resolution (Plargg, Dean of Chaos's
    /// "if a creature card was discarded this way"). Reset per resolution.
    CreatureCardsDiscardedThisEffect,
    /// Greatest mana value discarded this resolution, 0 if none
    /// (`GameState.greatest_discarded_mv_this_resolution`). Ill-Timed Explosion.
    GreatestDiscardedManaValueThisEffect,
    /// Creature cards among this resolution's `last_moved_cards` now in a
    /// graveyard — creature cards milled this way (Dread Summons).
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
    /// Distinct mana values among the cards in `who`'s hand (Atemsis,
    /// All-Seeing's "at least six different mana values").
    DistinctManaValuesInHand(PlayerRef),
    /// Distinct mana values among cards matching `filter` in `who`'s
    /// graveyard — Eris, Roar of the Storm ("for each different mana value
    /// among instant and sorcery cards in your graveyard").
    DistinctManaValuesInGraveyardMatching { who: PlayerRef, filter: SelectionRequirement },
    /// CR 903.8 — times `who` has cast their commander(s) from the command
    /// zone this game, summed over both partners (the commander-tax count;
    /// Thunderclap Drake's copies).
    CommanderCastsFromCommandZone(PlayerRef),
    /// CR 903.8 — times the commander(s) `what` names have been cast from the
    /// command zone this game, each partner on its own count. A cost static
    /// reads the spell being cast as `Selector::TriggerSource` (Myth Unbound).
    CommanderCastsOf(Box<Selector>),
    /// Greatest power among creatures the controller controls *and* creature
    /// cards in the controller's graveyard (0 if none). Ambitious Dragonborn
    /// enters with X +1/+1 counters equal to this.
    GreatestPowerControlledAndGraveyard,
    /// The greatest power among creatures `who` controls; 0 with no creatures
    /// (Season of Gathering's "draw cards equal to the greatest power among
    /// creatures you control").
    GreatestPowerControlled { who: PlayerRef },
    /// Mana value of the selector's first card, looked up on the battlefield and
    /// in graveyards, exile and hands. Wrath of the Skies.
    ManaValueOf(Box<Selector>),
    /// Highest mana value among the selector's permanents or cards in any zone; 0
    /// if none. Rush of Knowledge.
    HighestManaValueAmong(Box<Selector>),
    /// Number of distinct colors of the resolved permanent/card (CR 105.2 —
    /// "for each of its colors"). Reads printed colors; a colorless/devoid
    /// object counts 0. Breathe Your Last.
    ColorCountOf(Box<Selector>),
    /// How many creature types are noted for the permanents the selector
    /// resolves to (Volo, Itinerant Scholar's "for each creature type noted
    /// for target permanent you control named Volo's Journal").
    NotedCreatureTypesOf(Box<Selector>),
    /// CR 903.4 — colors in `who`'s commanders' color identity (the union for two
    /// commanders, CR 702.124c). War Room.
    ///
    /// ⚠ Zero for a seat with no commander, unlike
    /// `GameState::commander_identity_colors`, which answers all five so
    /// `ManaPayload::AnyColorInCommanderIdentity` still fixes in a cube. As a
    /// life *cost* that fallback would charge five.
    CommandersColorIdentityCount(PlayerRef),
    /// The greatest mana value among the player's commanders, wherever they
    /// are (CR 903.3 — a commander is a commander in every zone; CR 202.3 —
    /// printed mana value). Zero with no commander (Tangleweave Armor).
    GreatestCommanderManaValue(PlayerRef),
    /// Distinct colors across all the selector's entities (Case of the Shattered
    /// Pact). `ColorCountOf` reads one object.
    DistinctColorsAmong(Box<Selector>),
    /// Distinct planeswalker types among the selector's planeswalkers (CR
    /// 205.3j). Tam, the Possibility.
    DistinctPlaneswalkerTypesAmong(Box<Selector>),
    /// The number of colors among permanents the controller controls and
    /// spells they've cast this turn (First Family).
    ColorsAmongYoursAndSpellsCastThisTurn,
    /// The number of card types among spells the controller has cast this
    /// turn (April O'Neil, Hacktivist), off `Player::spell_casts_this_turn`.
    CardTypesAmongSpellsCastThisTurn,
    /// Converge: distinct colors of mana spent on the spell
    /// (`EffectContext.converged_value`, stashed at cast). Prismatic Ending.
    ConvergedValue,
    /// Distinct colors of mana spent on the resolving spell or ability (CR
    /// 601.2h / 602.2b; `EffectContext.mana_spent_by_color`). Unlike
    /// `ConvergedValue`, counts an activation's mana (Paragon of Modernity).
    DistinctColorsSpent,
    /// Mana from artifact sources spent to cast the source (Coin of Mastery's
    /// enters-with rider): the `CardData::cast_artifact_mana` stamp, so 0 for
    /// a permanent that wasn't cast.
    ArtifactManaSpentToCastSource,
    /// Mana from creatures spent to cast the trigger source — the spell a
    /// "whenever you cast" trigger saw (Inga and Esika's "three or more mana
    /// from creatures").
    CreatureManaSpentToCastTriggerSource,
    /// Mana from Deserts spent to cast the source spell (Cataclysmic
    /// Prospecting): the `CardData::cast_desert_mana` stamp.
    DesertManaSpentToCastSource,
    /// Mana from snow sources spent to cast the source spell — "the amount of
    /// {S} spent to cast this spell" (Blessing of Frost): the
    /// `CardData::cast_snow_mana` stamp, read off the pool's snow provenance
    /// (conservative, like the pool's own clamp).
    SnowManaSpentToCastSource,
    /// Mana from Treasures spent to cast the source spell — "a Treasure token
    /// for each mana from a Treasure spent to cast it" (Marut): the
    /// `CardData::cast_treasure_mana` stamp.
    TreasureManaSpentToCastSource,
    /// Excess: the distinct creatures `who` controlled that dealt combat damage
    /// to a player this turn, gone ones included (by controller on the
    /// battlefield, owner elsewhere).
    CreaturesThatCombatDamagedAPlayerThisTurn(PlayerRef),
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
    /// Mana symbols of `color` in the triggering spell's cost, hybrid / Phyrexian
    /// counted once as devotion does (CR 202.1). Namor the Sub-Mariner. 0 for
    /// non-cards.
    CastSpellPipsOf(crate::mana::Color),
    /// Total mana spent on the originating spell (`StackItem::Spell.mana_spent`,
    /// carried onto its cast triggers, read via `EffectContext.mana_spent`). SOS
    /// Increment and Opus payoffs.
    CastSpellManaSpent,
    /// Distinct card types in the top `count` of `who`'s library — Atraxa's
    /// reveal approximated as "draw N".
    DistinctTypesInTopOfLibrary { who: PlayerRef, count: Box<Value> },
    /// Number of distinct card types among the cards in `who`'s graveyard.
    /// Backs Broodspinner's "create that many Insects equal to the number of
    /// card types among cards in your graveyard" payoff.
    DistinctTypesInGraveyard { who: PlayerRef },
    /// Distinct card types among cards exiled with the source. Keen-Eyed Curator.
    DistinctCardTypesExiledWith,
    /// Distinct mana values among cards in exile stamped `exiled_with =
    /// source` (Azor's Gateway's "five or more different mana values").
    DistinctManaValuesExiledWith,
    /// Number of cards in exile stamped `exiled_with = source` (the resolving
    /// source). Backs "as long as three or more cards are exiled with this
    /// creature" static thresholds (Veteran Survivor).
    CardsExiledWithSourceCount,
    /// Untapped lands the active player controlled as this turn began
    /// (Power Surge).
    UntappedLandsActivePlayerHadAtTurnStart,
    /// Damage dealt to `who` this turn by artifact sources (Reverse Polarity).
    ArtifactDamageToPlayerThisTurn { who: PlayerRef },
    /// Half (rounded down) the damage dealt this turn by the highest-dealing
    /// sorcery spell `who` cast this turn. Backdraft; 0 if they cast none.
    HalfGreatestSorceryDamageThisTurn { who: PlayerRef },
    /// Number of distinct power values among creatures the controller
    /// controls. Backs Golden Ratio's "draw a card for each different
    /// power among creatures you control."
    DistinctPowerYouControl,
    /// The number of different kinds of counters among `who`'s permanents
    /// matching `filter`; a keyword counter is a kind (CR 122.1b). Perrie,
    /// the Pulverizer; Storm of Forms.
    CounterKindsAmong { who: PlayerRef, filter: crate::card::SelectionRequirement },
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
    /// Cards `who` has drawn this turn (`Player.cards_drawn_this_turn`). Fractal
    /// Anomaly.
    CardsDrawnThisTurn(PlayerRef),
    /// "for each card your opponents have drawn this turn" — the sum of
    /// `cards_drawn_this_turn` over the controller's living opponents
    /// (Heliod, the Warped Eclipse).
    CardsDrawnThisTurnByOpponents,
    /// Clues `who` has sacrificed this turn (`Player.clues_sacrificed_this_turn`;
    /// Briarbridge Patrol).
    CluesSacrificedThisTurn(PlayerRef),
    /// Cards `who` has drawn this step (`Player.cards_drawn_this_step`). Orcish
    /// Bowmasters' first-draw exemption.
    CardsDrawnThisStep(PlayerRef),
    /// Lands the player has played this turn (`Player.lands_played_this_turn`);
    /// `> 0` is landfall's "if a land entered" (Searing Blaze).
    LandsPlayedThisTurn(PlayerRef),
    /// Artifacts that entered the battlefield under the player's control this
    /// turn (`Player.artifacts_entered_this_turn`) — Malcator's end-step gate.
    ArtifactsEnteredThisTurn(PlayerRef),
    /// Tokens created under the player's control this turn
    /// (`Player.tokens_created_this_turn`) — Thalisse, Reverent Medium.
    TokensCreatedThisTurn(PlayerRef),
    /// Treasure tokens created under the player's control this turn
    /// (`Player.treasures_created_this_turn`) — Vazi, Keen Negotiator.
    TreasuresCreatedThisTurn(PlayerRef),
    /// Cards the player cycled this turn (`Player.cards_cycled_this_turn`) —
    /// Spellpyre Phoenix's "if you cycled two or more cards this turn".
    CardsCycledThisTurn(PlayerRef),
    /// Mounts and/or Vehicles that entered under the player's control this turn
    /// (`Player.mounts_vehicles_entered_this_turn`) — Cloudspire Coordinator's
    /// X token count.
    MountsVehiclesEnteredThisTurn(PlayerRef),
    /// Other creatures of the given type that entered under the controller this
    /// turn (Geralf, the Fleshwright).
    OtherCreaturesOfTypeEnteredThisTurn(crate::card::CreatureType),
    /// Nontoken creatures that entered under `who`'s control this turn
    /// (Gyome, Master Chef's end-step Food count).
    NontokenCreaturesEnteredThisTurn(PlayerRef),
    /// Every creature (token or not) that entered under `who`'s control this
    /// turn (Diamond City's "two or more creatures entered the battlefield
    /// under your control this turn").
    CreaturesEnteredThisTurn(PlayerRef),
    /// How many dice of the most recent roll showed natural face `.0` (Luck
    /// Bobblehead's "if you rolled 6 exactly seven times").
    LastRollFaceCount(u8),
    /// The number of distinct power values among creatures `0` controls
    /// ("one mana of that color for each different power among creatures you
    /// control" — Selvala, Eager Trailblazer).
    DistinctPowersAmongCreaturesControlled(PlayerRef),
    /// The player's poison counters (Vraska's −9 "counters equal to the
    /// difference" top-up).
    PoisonCountersOf(PlayerRef),
    /// The player's contract counters (Miss Highwater).
    ContractCountersOf(PlayerRef),
    /// CR 122.1f — the poison counters of every player `who` resolves to,
    /// summed (Vishgraz's "for each poison counter your opponents have").
    /// `PoisonCountersOf` reads one player.
    PoisonCountersAmong(PlayerRef),
    /// Rad counters summed over the players `who` names (Vault 12: "the
    /// total number of rad counters among players"; one player for a
    /// "has a rad counter" gate).
    RadCountersAmong(PlayerRef),
    /// Every counter every player has — poison, rad, energy, experience — the
    /// player half of "each counter among players and permanents"
    /// (Lumbering Megasloth).
    CountersAmongPlayers,
    /// How many of `who`'s players have `at_least` poison — Corrupted's count at
    /// a table (Wurmquake).
    PlayersWithPoisonAtLeast { who: PlayerRef, at_least: u32 },
    /// How many of the controller's opponents lost life this turn (Kaito,
    /// Bane of Nightmares' 0).
    OpponentsWhoLostLifeThisTurn,
    /// How many of the controller's opponents were dealt damage this turn
    /// (Gnoll War Band's discount). Backed by `Player.was_dealt_damage_this_turn`.
    OpponentsDealtDamageThisTurn,
    /// Living players (the controller included) who lost life this turn —
    /// Reaper's Scythe's "a soul counter for each player who lost life this
    /// turn".
    PlayersWhoLostLifeThisTurn,
    /// Multicolored spells the player has cast this turn (Zenith Chronicler).
    MulticoloredSpellsCastThisTurn(PlayerRef),
    /// The greatest total toxic value among creatures the player controls
    /// (Goliath Hatchery's Corrupted draw; CR 702.180b sums instances).
    GreatestToxicAmongControlled(PlayerRef),
    /// Two to the inner value, clamped at 30 to avoid overflow (SOS Mathemagics,
    /// "draws 2ˣ cards").
    Pow2(Box<Value>),
    /// Half of the inner value, rounded down. Used by SOS Pox Plague's
    /// "loses half their life", "discards half", "sacrifices half"
    /// clauses.
    HalfDown(Box<Value>),
    /// Permanents the resolved player controls — per-player reads inside a
    /// `ForEach` (Pox Plague), where `EachPermanent(ControlledByYou)` would read
    /// the spell's controller.
    PermanentCountControlledBy(PlayerRef),
    /// The filtered sibling of `PermanentCountControlledBy` — how many
    /// permanents the resolved player controls that match the filter
    /// (Mana Cache's "for each untapped land that player controls").
    PermanentCountControlledByMatching(PlayerRef, crate::card::SelectionRequirement),
    /// Number of players still in the game (not eliminated). "For each player"
    /// riders — Benediction of Moons's "gain 1 life for each player".
    PlayerCount,
    /// How many players `who` names (a fan-out like
    /// `OpponentsWhoVotedDifferently` counts its members) — Erestor's "scry X,
    /// where X is the number of opponents who voted for a choice you didn't".
    PlayersIn(PlayerRef),
    /// CR 700.11 — how many times the controller descended this turn (permanent
    /// cards put into their graveyard). The Mycotyrant's end-step token count.
    TimesDescendedThisTurn,
    /// Creatures the resolved player controls, for per-player `ForEach` reads
    /// (Biorhythm).
    CreatureCountControlledBy(PlayerRef),
    /// The largest group of the controller's creatures sharing a creature type;
    /// changelings count for every type. White Lotus Tile.
    GreatestSharedCreatureTypeCount,
    /// Creatures the evaluating player controls that share a creature type
    /// with `.0` (Mana Echoes). Changelings share with everything.
    CreaturesSharingTypeWith(Box<Selector>),
    /// Nonbasic lands the resolved player controls, read per recipient inside a
    /// `ForEach` (Sunspine Lynx).
    NonbasicLandCountControlledBy(PlayerRef),
    /// Loyalty counters on the selector's first permanent; 0 for anything else
    /// (Confront the Past).
    LoyaltyOf(Box<Selector>),
    /// The amount on the event that fired this trigger (life gained, damage,
    /// cards drawn, …), from `EffectContext.event_amount`. Light of Promise. 0
    /// outside a trigger.
    TriggerEventAmount,
    /// CR 605.1a / 106.4 — the mana the `TappedForMana` event's permanent
    /// produced in the trigger's batch, one per pip event (Tyvar the
    /// Bellicose's "equal to the amount of mana this creature produced").
    TriggerManaProduced,
    /// Auras the trigger's controller controlled attached to the dying subject
    /// when it left (`GameState.auras_at_death`). Hateful Eidolon.
    AurasYouControlledOnDyingSubject,
    /// CR 603.10a — attachments on the dying subject when it left that match
    /// `filter`, wherever they are now (`GameState.auras_at_death`). Gunner
    /// Conscript's "if it was enchanted / equipped".
    AttachmentsOnDyingSubject { filter: SelectionRequirement },
    /// The number of spells `who` has cast this turn from anywhere other than
    /// their hand (Paradox — Impending Flux, Surge of Brilliance):
    /// `spells_cast_this_turn - spells_cast_from_hand_this_turn`.
    SpellsCastNotFromHandThisTurn(PlayerRef),
    /// The number of permanents matching `filter` attached to the permanent
    /// `what` resolves to — "as long as this creature is enchanted by exactly
    /// two Auras" (Timber Paladin).
    AttachmentsOn { what: Box<Selector>, filter: SelectionRequirement },
    /// Cards in exile linked to the source (`exiled_with`) matching `filter` —
    /// Soulflayer's delved creature cards.
    CardsExiledWithSourceMatching(SelectionRequirement),
    /// CR 706.4 — the most recent die roll this resolution, set before each
    /// result-table arm runs (Ancient Copper Dragon). 0 outside a roll.
    LastDieRoll,
    /// CR 706.4 — how many different results the most recent roll showed
    /// ("for each different result" — Circuits Act).
    LastRollDistinctResults,
    /// Creatures that died under `who`'s control this turn
    /// (`Player.creatures_died_this_turn`). Predicate sibling:
    /// `Predicate::CreaturesDiedThisTurnAtLeast`.
    CreaturesDiedThisTurn(PlayerRef),
    /// Creatures that left the battlefield under `who`'s control this turn,
    /// by any route (Kutzil's Flanker). `Player.creatures_left_battlefield_count`.
    CreaturesLeftBattlefieldThisTurn(PlayerRef),
    /// Number of permanents `who` has sacrificed so far this turn. Backed by
    /// `Player.permanents_sacrificed_this_turn`.
    PermanentsSacrificedThisTurn(PlayerRef),
    /// Artifacts and/or creatures `who` has sacrificed this turn. Backed by
    /// `Player.artifacts_or_creatures_sacrificed_this_turn` (Dargo).
    ArtifactsOrCreaturesSacrificedThisTurn(PlayerRef),
    /// Card types among permanents `who` sacrificed this turn (Korvold,
    /// Gleeful Glutton's "{1} less for each card type").
    CardTypesSacrificedThisTurn(PlayerRef),
    /// Creatures that died this turn across every seat; mirrors
    /// `Predicate::CreaturesDiedThisTurnTotalAtLeast`.
    CreaturesDiedThisTurnTotal,
    /// Number of creatures that died **under the controller's control** this
    /// turn (`Player.creatures_died_this_turn` for `ctx.controller`). Liliana's
    /// Standard Bearer.
    ControllerCreaturesDiedThisTurn,
    /// "The greatest amount of damage dealt by a source to a permanent or
    /// player this turn" (Impact Resonance): the largest total one source
    /// dealt one permanent or player, the victim gone or not (2014-11-07).
    GreatestDamageFromOneSourceThisTurn,
    /// "Remove any number of counters" as a cast cost (Hierophant
    /// Bio-Titan): the count the casting seat chose, else `default` — the
    /// headless pick, which the cost and its discount both read.
    CastCounterRemovalOr(Box<Value>),
    /// Damage the resolving source has dealt this turn (Chandra, Fire of
    /// Kaladesh's "if Chandra has dealt 3 or more damage this turn").
    DamageDealtBySourceThisTurn,
    /// "The greatest power among creatures sacrificed this way" — the largest
    /// power among the permanents sacrificed during this resolution, read off
    /// their death snapshots (Shadowgrange Archfiend).
    GreatestSacrificedPowerThisResolution,
    /// Creatures that died this turn whose death snapshot matches `filter`, with
    /// the controller as "you" (Caller of the Claw: `NotToken.and(OwnedByYou)`).
    CreatureDeathsThisTurnMatching { filter: SelectionRequirement },
    /// The total power (last known) of the creatures matching `filter` that
    /// died this turn — "the total power of Daleks that died this turn"
    /// (Genesis of the Daleks IV).
    CreatureDeathsThisTurnTotalPower { filter: SelectionRequirement },
    /// Number of Zubera that died this turn across **every** player. Sums
    /// `Player.zuberas_died_this_turn`. Powers the Champions-of-Kamigawa
    /// Zubera death-trigger cycle ("for each Zubera that died this turn").
    ZuberasDiedThisTurnTotal,
    /// Permanents `Effect::Destroy` destroyed earlier this resolution
    /// (`permanents_destroyed_this_resolution`). Culling Ritual.
    PermanentsDestroyedThisResolution,
    /// Creatures (tokens too) destroyed earlier this resolution that `who`
    /// controlled as they died (Deadly Tempest).
    CreaturesDestroyedThisResolutionControlledBy(PlayerRef),
    /// As `CreaturesDestroyedThisResolutionControlledBy`, tokens excluded —
    /// Ceaseless Conflict's "for each nontoken creature you controlled that
    /// was destroyed this way".
    NontokenCreaturesDestroyedThisResolutionControlledBy(PlayerRef),
    /// Permanents of any type destroyed this resolution that `who` controlled
    /// (From the Ashes); same scratch as
    /// `CreaturesDestroyedThisResolutionControlledBy`.
    PermanentsDestroyedThisResolutionControlledBy(PlayerRef),
    /// Nonartifact creatures destroyed earlier in this resolution, tokens
    /// included (Join the Collective's Borg).
    NonartifactCreaturesDestroyedThisResolution,
    /// Snow permanents (CR 205.4g) the resolved player controls. Skred.
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
    /// CR 123.6e — the unique vowels on the name sticker the most recent
    /// `Effect::PutNameSticker` in this resolution put on something; 0 when
    /// it put none ("Add {R} for each unique vowel on that sticker").
    NameStickerUniqueVowels,
    /// [`Value::GreatestCommanderManaValue`] counting only commanders on the
    /// battlefield or in the command zone (Imposing Grandeur).
    GreatestCommanderManaValueInPlayOrCommandZone(PlayerRef),
    /// [`Value::DungeonsCompleted`] counting each dungeon name once —
    /// "each differently named dungeon you've completed" (Ellywick's emblem).
    DistinctDungeonsCompleted,
    /// Cards put into `who`'s graveyard from their library this turn, milled
    /// or surveiled (CR 701.42a) — Cruel Calculations' X.
    CardsPutIntoGraveyardFromLibraryThisTurn(PlayerRef),
}

impl Value {
    pub const ZERO: Value = Value::Const(0);
    pub const ONE: Value = Value::Const(1);
    pub fn count(sel: Selector) -> Self { Value::CountOf(Box::new(sel)) }
}

// ── Predicate ────────────────────────────────────────────────────────────────

/// `Predicate::SourceIsSameObjectOnBattlefield`'s "bind me to the stamp this
/// object is about to get as it enters" value.
pub const UNBOUND_OBJECT_STAMP: u64 = u64::MAX;

/// The event batches [`Predicate::FirstBatchThisTurn`] counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FirstBatchKind {
    TokensCreated,
    Surveiled,
    /// "Whenever you lose life for the first time each turn" (Gonti's
    /// Machinations).
    LifeLost,
}

/// A boolean game-state condition (for `Effect::If` / cast-time checks).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Predicate {
    #[default]
    True,
    False,
    Not(Box<Predicate>),
    All(Vec<Predicate>),
    Any(Vec<Predicate>),
    /// `pred` holds for some player `who` names, each bound as `Triggerer` — "if
    /// an opponent [did X]" (Mindbreak Trap). A single-player predicate given
    /// `EachOpponent` reads only the first seat (CR 800.4).
    ForAnyPlayer { who: PlayerRef, pred: Box<Predicate> },
    /// A `kind` counter put on `what` (one permanent) now would land: no
    /// Solemnity lock and no restriction (Blightbeetle) zeroes it. CR
    /// 702.100b — a creature evolves only if the counter was put.
    CounterWouldLand { what: Selector, kind: crate::card::CounterType },
    /// Squall, Gunblade Duelist: a creature attacking the trigger's player has
    /// power or toughness equal to the source's `chosen_number`.
    AttackerOfTriggererPtEqualsSourceChosenNumber,
    /// At least one entity matches the selector.
    SelectorExists(Selector),
    /// Selector resolves to at least `n` entities.
    SelectorCountAtLeast { sel: Selector, n: Value },
    /// Every battlefield permanent matching `filter` shares at least one
    /// colour with all the others (Common Cause). Vacuously true when nothing
    /// matches; false as soon as one match is colourless.
    AllMatchingShareAColor(crate::card::SelectionRequirement),
    /// Two of the objects `.0` resolves to share a card type (Rowdy Crew's "if
    /// two cards that share a card type are discarded this way").
    TwoShareACardType(Selector),
    /// A card `who` discarded earlier in this resolution shares a card type
    /// with a card `with` discarded in it (Creeping Dread's "each opponent who
    /// discarded a card that shares a card type with the card you discarded").
    DiscardedSharesCardTypeWith { who: PlayerRef, with: PlayerRef },
    /// An opponent of the controller was dealt noncombat damage last turn
    /// (`Player.dealt_noncombat_damage_last_turn` — Command the Stage).
    OpponentDealtNoncombatDamageLastTurn,
    /// `who` tapped a land for mana this turn (Desolation).
    TappedLandForManaThisTurn(PlayerRef),
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
    /// The target in slot `slot` (stack spell or permanent) shares a colour with
    /// a `filter` permanent you control (Jaded Response).
    TargetSharesColorWithControlled { slot: u8, filter: crate::card::SelectionRequirement },
    /// The permanent in target `slot` shares a colour with the mana the
    /// trigger's tap produced (a `TappedForMana` trigger's event amount is
    /// that `ColorSet`): C.A.M.P.
    TargetSharesColorWithTappedMana { slot: u8 },
    /// True if `who` sacrificed at least one permanent during the current
    /// resolution. Backed by `GameState::players_sacrificed_this_resolution`.
    /// Gates "if you sacrificed a permanent this way, …" (Deadly Brew).
    PlayerSacrificedThisResolution(PlayerRef),
    /// CR 120.10 — excess damage was dealt this resolution
    /// (`GameState.excess_damage_this_resolution`). Orbital Plunge.
    ExcessDamageDealtThisResolution,
    /// The controller has cast a spell with the source's printed name at least
    /// `n` times this game. The resolving cast counts, so "another spell named X"
    /// is `n = 2` (Approach of the Second Sun).
    CastOwnNameThisGameAtLeast(u32),
    /// At least `n` mode instances were chosen for this cast (`ctx.spree_modes`).
    /// With distinct modes and `max = n` this is "if you chose all of the above"
    /// (Multiple Choice); a single-mode cast fails it.
    ChoseModesAtLeast(u8),
    /// It's `who`'s turn.
    IsTurnOf(PlayerRef),
    /// CR 305.2 / 305.3 — it's `who`'s turn and they have a land play left
    /// (an effect that says "play" a land during its resolution).
    CanPlayLandNow(PlayerRef),
    /// CR 800.4 — at least this many players have lost the game (Hot
    /// Pursuit's "if two or more players have lost the game").
    PlayersLostAtLeast(u8),
    /// The permanent returned to hand to pay this activation's cost had a
    /// nonbasic land type as it last existed on the battlefield (Wonderscape
    /// Sage).
    CostReturnedHadNonbasicLandType,
    /// The event's card (a spell just cast, a land just played) was one of
    /// the cards exiled with the source — Share the Spoils' "when they do".
    TriggerCardExiledWithSource,
    /// CR 702.143 — "if this spell was foretold": the resolving spell was
    /// cast for its foretell cost (Haunting Voyage).
    CastForetold,
    /// `who` resolves to a player who is an opponent of the source's
    /// controller ("…deals combat damage to one of your opponents" —
    /// Gonti, Night Minister). Teammates and the controller are false.
    PlayerIsOpponent { who: PlayerRef },
    /// During the controller's main phase. An instant resolves in the step it was
    /// cast, so this captures "if you cast this during your main phase".
    YourMainPhase,
    /// The active player controls something the selector resolves to — "at the
    /// beginning of the upkeep of enchanted permanent's controller" Auras (Warp
    /// Artifact) pair it with an `AnyPlayer` upkeep trigger.
    ActivePlayerControls(Box<Selector>),
    /// The game is currently in the given turn step (CR 500). Gates
    /// "activate only during [your] upkeep / end step" abilities; pair with
    /// `IsTurnOf(You)` for the "your" qualifier.
    CurrentStepIs(crate::turn_step::TurnStep),
    /// The given entity's properties match the filter.
    EntityMatches { what: Selector, filter: SelectionRequirement },
    /// Some entity of `what` matches `filter`; false when empty, unlike the
    /// vacuous `EntityMatches`. For "up to one" targets (Diregraf Scavenger).
    EntityMatchesAny { what: Selector, filter: SelectionRequirement },
    /// `who` has gained at least `at_least` total life this turn.
    /// Backed by `Player.life_gained_this_turn`. Used by Strixhaven's
    /// **Infusion** rider — "If you gained life this turn, …".
    LifeGainedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has not yet completed a life-gain trigger batch this turn
    /// (`Player.gained_life_earlier_this_turn`): "the first time you gain life
    /// each turn" (Leech Collector). Pair with `once_per_turn()`.
    FirstLifeGainThisTurn { who: PlayerRef },
    /// The trigger's batch is `who`'s first of `kind` this turn, counting batches
    /// from before the listener arrived (Akim, the Soaring Wind). Backed by
    /// `GameState::tally_first_each_turn_batches`.
    FirstBatchThisTurn { who: PlayerRef, kind: FirstBatchKind },
    /// "Activate only if you've scried or surveilled this turn" (Proctor of
    /// Potential, CR 701.22 / 701.25). Backed by the per-turn scry and
    /// surveil batch tallies.
    ScriedOrSurveiledThisTurn { who: PlayerRef },
    /// "If you've played a land … this turn from anywhere other than your
    /// hand" (Spider-Man 2099) — `Player.played_land_not_from_hand_this_turn`.
    PlayedLandNotFromHandThisTurn { who: PlayerRef },
    /// "A spell that targets only a single [filter]" (Precursor Golem): the
    /// spell `what` names has exactly one target, and it matches `filter`.
    SpellTargetsOnlyOne { what: Selector, filter: crate::card::SelectionRequirement },
    /// CR 700.14 — this cast's payment takes the active player's mana spent on
    /// spells this turn from below `n` to at least `n`. The filter on
    /// `EventKind::Expend` triggers (Roughshod Duo).
    ExpendReached(u32),
    /// Among this resolution's discards, the controller discarded one with the
    /// greatest mana value (ties count) — Cait, Cage Brawler.
    YouDiscardedGreatestManaValueThisEffect,
    /// CR 700.6 — `who` has the city's blessing. "As long as you have the
    /// city's blessing, …" (Ascend payoffs).
    HasCityBlessing { who: PlayerRef },
    /// `who` has discarded a card this turn (Asmoranomardicadaistinaculdacar's
    /// alternative-cost gate). Reads `Player.discarded_this_turn`.
    DiscardedThisTurn { who: PlayerRef },
    /// CR 732 — it's currently day.
    IsDay,
    /// CR 732 — it's currently night.
    IsNight,
    /// CR 706.2 — true when the firing die result (`Performed(DieResult)`) is
    /// at least `n` — or, off a whole-roll `RolledDice`, its greatest result —
    /// via `EffectContext.event_amount` (Ground Pounder, Mr. House).
    DieResultAtLeast(u8),
    /// The evaluating source was marked by `Effect::MarkSourceThisTurn` this
    /// turn.
    SourceMarkedThisTurn,
    /// CR 702.85a — the spell this `SpellCast` trigger fired on has cascade,
    /// printed or granted by a cast trigger that holds for it. Aurora Phoenix.
    CastSpellHasCascade,
    /// CR 726 — `who` is the monarch ("as long as you're the monarch, …").
    IsMonarch { who: PlayerRef },
    /// CR 726 — `who` was the monarch as the turn began (turn-start snapshot).
    /// Knights of the Black Rose.
    WasMonarchAtTurnStart { who: PlayerRef },
    /// CR 727 — `who` currently has the initiative.
    HasInitiative { who: PlayerRef },
    /// CR 903 — `who` controls their own commander (either of two); Lieutenant's
    /// condition. A stolen commander doesn't count.
    ControlsOwnCommander { who: PlayerRef },
    /// `who` controls `at_least` lands that share one name (Endless Atlas —
    /// "three or more lands with the same name"): the largest same-name
    /// group, not a total across names.
    ControlsLandsWithSameNameAtLeast { who: PlayerRef, at_least: u32 },
    /// The filtered sibling: `who` controls `at_least` permanents matching
    /// `filter` that share one name (Mechanized Production — "eight or more
    /// artifacts with the same name as one another").
    ControlsSameNamedAtLeast { who: PlayerRef, filter: SelectionRequirement, at_least: u32 },
    /// Mana from a Treasure was spent to cast the spell `what` resolves to
    /// (Alchemist's Talent, Rain of Riches — on a `SpellCast` trigger,
    /// `Selector::TriggerSource`).
    CastWithTreasureMana { what: Selector },
    /// Snow mana of one of the spell's own colours was spent on `what`
    /// (`CardData::cast_snow_colors`). Boreal Outrider.
    SnowOfItsColorSpentToCast { what: Selector },
    /// `SpendRestriction::MarksCast(mark)` mana was spent to cast `what`
    /// (Barracks of the Thousand's "using mana produced by" this).
    CastWithMarkedMana {
        what: Selector,
        #[serde(default)]
        mark: u8,
    },
    /// CR 702.179 — `who`'s speed is at least `speed` (0–4). "Max speed —"
    /// abilities use `speed: 4`; "as long as your speed is N or higher" uses
    /// the listed threshold. Backed by `Player.speed`.
    SpeedAtLeast { who: PlayerRef, speed: u32 },
    /// Some `who` player was dealt damage this turn
    /// (`Player.was_dealt_damage_this_turn`). Bloodthirst (CR 702.54) with
    /// `EachOpponent`.
    PlayerDamagedThisTurn { who: PlayerRef },
    /// True if a creature or planeswalker any player matched by `who`
    /// controlled was dealt excess damage this turn (CR 120.10 — Rith).
    ExcessDamagedPermanentThisTurn { who: PlayerRef },
    /// The source dealt damage to another creature this turn, that creature
    /// alive or not (Wolverine, Best There Is).
    SourceDamagedAnotherCreatureThisTurn,
    /// Any player matched by `who` has an enduring story (storied).
    HasEnduringStory { who: PlayerRef },
    /// True if any player matched by `who` has lost life this turn (damage or
    /// direct life loss). Backed by `Player.lost_life_this_turn`. Powers
    /// Spectacle (CR 702.137) — pair with `who: EachOpponent`.
    PlayerLostLifeThisTurn { who: PlayerRef },
    /// True if any player matched by `who` lost life during the previous turn
    /// ("if you lost life last turn" — Paladin of Atonement). Backed by
    /// `Player.lost_life_last_turn`.
    PlayerLostLifeLastTurn { who: PlayerRef },
    /// True if a permanent was put into the hand of any player matched by
    /// `who` from the battlefield this turn. Backed by
    /// `Player.permanent_to_hand_this_turn` (Barrin, Tolarian Archmage).
    PermanentPutIntoHandThisTurn { who: PlayerRef },
    /// True if any player matched by `who` has gained life this turn. Backed by
    /// `Player.life_gained_this_turn`. Powers "if you gained or lost life this
    /// turn" end-step payoffs (Starlit Soothsayer, Star Charter).
    PlayerGainedLifeThisTurn { who: PlayerRef },
    /// Some `who` player has drawn `n`+ cards this turn. With `Triggerer` +
    /// `once_per_turn`: "draws their Nth card each turn" (Faerie Mastermind).
    PlayerDrewAtLeastThisTurn { who: PlayerRef, n: u32 },
    /// Suppressor Skyguard: the attacking player has a live opponent other than
    /// you whom no attacker is aimed at.
    AttackingPlayerLeftAnotherOpponentUnattacked,
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
    /// Some `who` player has life at least `delta` above their starting total (CR
    /// 103.4). Righteous Valkyrie.
    PlayerLifeAtLeastAboveStarting { who: PlayerRef, delta: i32 },
    /// Some `who` player has at most half their starting life (rounded down).
    /// Cecil, Dark Knight.
    PlayerLifeAtMostHalfStarting { who: PlayerRef },
    /// Some `who` player has (or ties for) the most life — Dethrone (CR 702.105)
    /// with `DefendingPlayer`.
    PlayerHasMostLife { who: PlayerRef },
    /// No opponent of the evaluating controller has more life than `who`
    /// (Veteran Soldier's "if no opponent has more life than that player" —
    /// the controller's own total does not count, unlike `PlayerHasMostLife`).
    PlayerHasMostLifeAmongOpponents { who: PlayerRef },
    /// The controller controls a permanent of the source's chosen color (Lurebound
    /// Scarecrow); friendly, so the as-enters pick names your densest color.
    YouControlPermanentOfSourcesChosenColor,
    /// `who` controls strictly more `filter` permanents than each other player
    /// (Rivalry's land leader, Damping Engine's permanent leader).
    PlayerControlsMostOf { who: PlayerRef, filter: SelectionRequirement },
    /// `value` read with `You` = `who` beats it read for every other live player
    /// (Battle Angels of Tyr).
    PlayerValueExceedsEachOther { who: PlayerRef, value: Value },
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
    /// The trigger's subject has `color` — a battlefield permanent's current
    /// colors, or the resolving spell's (a card in no zone mid-resolution):
    /// "whenever a red source you control deals damage" (The Red Terror).
    TriggerSourceHasColor(crate::mana::Color),
    /// The trigger's subject is the permanent the effect's source is attached
    /// to — "whenever enchanted creature becomes tapped" (Stinging Licid).
    TriggerSourceIsSourceHost,
    /// True if the effect's source creature attacked this turn (CR 702.142
    /// Boast gate). Backed by `CardInstance.attacked_this_turn`.
    SourceAttackedThisTurn,
    /// The top of the stack is an ability with one target, the source (Agrus Kos,
    /// Eternal Soldier), asked as it is announced.
    AbilityOnStackTargetsOnlySource,
    /// CR 500.7 — the current turn is an extra turn. Backed by
    /// `GameState.current_turn_is_extra`. Medomai the Ageless's
    /// "can't attack during extra turns".
    IsExtraTurn,
    /// CR 701.37 — the source permanent is monstrous. Backed by
    /// `CardInstance.monstrous`. Powers "as long as this is monstrous, …"
    /// statics (Fleecemane Lion's hexproof + indestructible).
    SourceIsMonstrous,
    /// CR 702.112 — the source permanent is renowned. Backed by
    /// `CardInstance.renowned`. Powers "if it's renowned, …" attack intervening-
    /// ifs (Consul's Lieutenant) and "as long as renowned" statics.
    SourceIsRenowned,
    /// CR 701.64b / 702.186b — the source permanent is harnessed, so its ∞
    /// abilities work (The Soul Stone's upkeep reanimation).
    SourceIsHarnessed,
    /// CR 301.5 — the source permanent is equipped (an Equipment is attached).
    /// Powers "as long as this creature is equipped, …" team statics (Auriok
    /// Steelshaper's Soldier/Knight anthem).
    SourceIsEquipped,
    /// The source is a creature by computed types (an animated manland).
    /// Wandering Fumarole.
    SourceIsCreature,
    /// The ability's source permanent is still on the battlefield — the
    /// intervening-'if' half of "…if [this] is still on the battlefield"
    /// (Shirei, Shizo's Caretaker's delayed return).
    SourceOnBattlefield,
    /// The trigger's leaving permanent had one or more counters on it as it
    /// left (CR 603.10 — its last-known information; Nikara, Lair Scavenger).
    TriggerSourceHadCounters,
    /// The same last-known read for one counter kind — "for as long as that
    /// creature has a bounty counter on it, it has 'when this dies …'"
    /// (Mathas, Fiend Seeker).
    TriggerSourceHadCounter(crate::card::CounterType),
    /// An opponent of the controller had at least `n` creatures enter under
    /// their control this turn (Whiplash Trap's alternative cost).
    AnOpponentHadCreaturesEnterAtLeast(u32),
    /// CR 400.7 — the source is on the battlefield as the same object a delayed
    /// trigger was scheduled for (its `battlefield_timestamp` is unchanged; a
    /// `CardId` alone survives leaving and returning — Hellkite Courser).
    /// Schedule with [`UNBOUND_OBJECT_STAMP`] as the object enters; the entry
    /// stamp binds it (`rebind_object_stamp`), and attach / transform /
    /// turn-face-up re-stamps carry it.
    SourceIsSameObjectOnBattlefield { battlefield_timestamp: u64 },
    /// CR 400.7 — the delayed body's `Target(0)` is still the captured object
    /// (`battlefield_timestamp` unchanged), so a card that came back and left
    /// again doesn't match. Stamped by `Effect::DelayUntilWithCapture`.
    TargetIsCapturedObject { battlefield_timestamp: u64 },
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
    /// No permanent left the battlefield this turn, under anyone's control
    /// (Sarevok, Deathbringer) — every player's
    /// `permanent_left_battlefield_this_turn` is clear.
    NoPermanentLeftBattlefieldThisTurn,
    /// The controller has cast no spell this game, read before the cast counts —
    /// "this is the first spell you've cast" (Once Upon a Time).
    FirstSpellYouCastThisGame,
    /// A creature left the battlefield under `who`'s control this turn (Tale
    /// of Momo's cost reduction). Backed by
    /// `Player.creature_left_battlefield_this_turn`.
    CreatureLeftBattlefieldThisTurn { who: PlayerRef },
    /// `who` has airbent, earthbent, firebent and waterbent this turn (Avatar
    /// Aang's "if you've done all four this turn"). Backed by
    /// `Player.bends_this_turn`.
    BentAllFourThisTurn { who: PlayerRef },
    /// EOE Void: a nonland permanent left the battlefield this turn, or `who`
    /// cast a spell for warp this turn.
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
    /// The source reached the battlefield by being cast (any path), not as a
    /// token or by an effect — "if you cast it" ETB riders (Skitterbeam
    /// Battalion). Backed by `CardInstance` cast flags.
    SourceWasCast,
    /// `SourceWasCast` asked of the trigger's subject — "whenever an Aura you
    /// control enters, if you cast it" (Light-Paws, Emperor's Voice).
    TriggerSourceWasCast,
    /// The source was cast from its owner's hand
    /// (`CardInstance.cast_from_hand`) — Phage the Untouchable.
    SourceCastFromOwnersHand,
    /// CR 702.77 — true if a card in exile is champion-linked to the
    /// effect's source (`exiled_by` points at it). Gates "when a [type] is
    /// championed with this creature" riders (Mistbind Clique).
    SourceChampionedSomething,
    /// True if the trigger's subject (a blocker) is blocking the listening
    /// source. Scopes a `Blocks`/`AnyPlayer` trigger to "whenever a creature
    /// blocks *this* creature" (Nessian Boar).
    TriggerBlocksSource,
    /// The just-cast spell has the name the source stamped via `Effect::NameCard`
    /// (Silverquill Silencer).
    TriggerObjectNameMatchesNamedCard,
    /// The trigger object (`trigger_source`) has the source's
    /// `chosen_creature_type` (changelings match) — "whenever you cast a spell of
    /// the chosen type" (Vanquisher's Banner).
    TriggerObjectIsChosenType,
    /// True if any player `who` resolves to attacked with a creature this
    /// turn (Raid, CR 702.108 ability word). Backed by
    /// `Player.attacked_this_turn`.
    PlayerAttackedThisTurn { who: PlayerRef },
    /// True if the controller attacked any player `who` resolves to this turn
    /// (CR 508.1 — "for each opponent you attacked this turn": Rite of
    /// Ascension under `ForEachOpponent`). Off the controller's turn, false.
    YouAttackedPlayerThisTurn { who: PlayerRef },
    /// True if `who` has paid or lost at least `n` {E} this turn. Backed by
    /// `Player.energy_spent_this_turn` (Izzet Generatorium's "{T}: Draw a card.
    /// Activate only if you've paid or lost four or more {E} this turn").
    EnergyPaidThisTurnAtLeast { who: PlayerRef, n: u32 },
    /// True if a creature `who` controlled dealt combat damage to a player
    /// this turn. Backed by `Player.dealt_combat_damage_to_player_this_turn`.
    DealtCombatDamageToPlayerThisTurn { who: PlayerRef },
    /// CR 702.173a — a player was dealt combat damage this turn by an Assassin
    /// creature or a commander `who` controlled (Freerunning's alt-cost gate).
    /// Backed by `Player.freerunning_unlocked_this_turn`.
    FreerunningUnlocked { who: PlayerRef },
    /// The effect's source permanent has dealt damage since it entered —
    /// negated, "as long as [this] hasn't dealt damage yet" (Ratonhnhaké꞉ton).
    SourceHasDealtDamage,
    /// The source permanent has dealt **combat** damage since it entered —
    /// negated, "as long as [this] hasn't dealt combat damage yet" (Ruric
    /// Thar, Magecrusher). Shares `CardInstance::has_dealt_damage`.
    SourceHasDealtCombatDamage,
    /// `who` was attacked this turn by an Assassin the effect's controller
    /// controlled (Ramses, Assassin Lord). Backed by
    /// `Player::attacked_by_assassins_of`.
    AttackedByYourAssassinThisTurn { who: PlayerRef },
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
    /// "[who] cast a spell with none of `types` this turn" — a nonartifact
    /// spell (The Fifteenth Doctor's "the first nonartifact spell you cast
    /// each turn"). Reads `Player.spell_casts_this_turn`.
    CastSpellThisTurnWithout {
        who: PlayerRef,
        types: Vec<crate::card::CardType>,
    },
    /// "[who] cast a spell with mana value `at_least` or greater this turn"
    /// (Rhino, Barreling Brute). Reads `Player.spell_casts_this_turn`.
    CastSpellWithManaValueAtLeastThisTurn { who: PlayerRef, at_least: u32 },
    /// `who` has cast at most `n` spells of `card_type` this turn, the spell
    /// being cast included — "whenever you cast your first enchantment spell
    /// each turn" is `n: 1` as a cast-trigger filter (Tuvasa the Sunlit).
    SpellsOfTypeCastThisTurnAtMost { who: PlayerRef, card_type: crate::card::CardType, n: u32 },
    /// "[who] has been dealt damage by `at_least` different creatures this
    /// turn" (Inferno Trap). Reads `Player.creatures_that_damaged_me_this_turn`.
    DamagedByCreaturesThisTurnAtLeast { who: PlayerRef, at_least: u32 },
    /// "[who] had `at_least` lands enter the battlefield under their control
    /// this turn" (Lavaball Trap). Reads `Player.lands_entered_this_turn`.
    LandsEnteredThisTurnAtLeast { who: PlayerRef, at_least: u32 },
    /// A `filter` creature entered under `who` this turn
    /// (`Player.creatures_entered_this_turn`; departed ones count). Permafrost
    /// Trap.
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
    /// Any player `who` resolves to discarded a nonland card this resolution
    /// (`scratch.nonland_cards_discarded_per_player_this_resolution`). Kroxa:
    /// `Not(..{ Triggerer })` under a per-opponent `ForEach`.
    DiscardedNonlandThisEffect { who: PlayerRef },
    /// `who` discarded a card this resolution
    /// (`scratch.cards_discarded_per_player_this_resolution`). Fanatic of the
    /// Harrowing.
    DiscardedThisEffect { who: PlayerRef },
    /// The most recently discarded card (cost or effect) had this creature
    /// type — Necromancer's Stockpile's "if the discarded card was a Zombie".
    /// Backed by `GameState.last_discarded_creature_types`.
    LastDiscardedHasCreatureType(crate::card::CreatureType),
    /// At least `at_least` cards left `who`'s graveyard this turn
    /// (`Player.cards_left_graveyard_this_turn`). Lorehold payoffs (Living
    /// History).
    CardsLeftGraveyardThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// [`Predicate::CardsLeftGraveyardThisTurnAtLeast`] for creature cards only
    /// (Syrix, Carrier of the Flame).
    CreatureCardsLeftGraveyardThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// True if an opponent of `who` has cast a spell since `who`'s last turn
    /// ended (I Bask in Your Silent Awe's abandon check). Reads the
    /// `GameState.opponent_cast_since_your_turn` seat mask, cleared at `who`'s
    /// cleanup.
    OpponentCastSpellSinceYourTurn { who: PlayerRef },
    /// True if `who` has been dealt combat damage since their last turn ended
    /// (Marchesa, Resolute Monarch). `GameState.combat_damaged_since_your_turn`.
    CombatDamagedSinceYourTurn { who: PlayerRef },
    /// CR 701.19 — true if `who` (any matching player) searched their own
    /// library this turn. `PlayerRef::EachOpponent` is satisfied by any one
    /// opponent. Archive Trap's "rather than pay" gate.
    SearchedLibraryThisTurn { who: PlayerRef },
    /// CR 702.76 Prowl: a controller's creature of a listed type dealt combat
    /// damage to a player this turn (`Player.prowl_types_this_turn`).
    ProwlTypeDealtCombatDamage { types: Vec<crate::card::CreatureType> },
    /// [`ProwlTypeDealtCombatDamage`](Self::ProwlTypeDealtCombatDamage) under
    /// ANY player's control: "if a player was dealt combat damage by a Zombie
    /// this turn" (Lost Monarch of Ifnir).
    TypeDealtCombatDamageByAnyone { types: Vec<crate::card::CreatureType> },
    /// Some `who` player had `at_least` cards put into their graveyard this turn
    /// (CR 700.4 tally). Ravenous Trap.
    CardsToGraveyardThisTurnAtLeast { who: PlayerRef, at_least: u32 },
    /// "[N] or more creature cards were put into graveyards from anywhere this
    /// turn" (Case of the Gorgon's Kiss). Counts every player's tally.
    CreatureCardsToGraveyardThisTurnAtLeast(u32),
    /// "[N] or more sources you controlled dealt damage this turn" (Case of
    /// the Burning Masks). Counts distinct sources per controller-at-the-time.
    SourcesYouControlledDealtDamageThisTurnAtLeast(u32),
    /// "If a source you controlled dealt [N] or more damage this turn"
    /// (Dragon Cultist): one source's damage while you controlled it.
    SourceYouControlledDealtDamageThisTurnAtLeast(u32),
    /// `who` has cast at least `at_least` spells this turn
    /// (`Player.spells_cast_this_game_turn`). Burrog Barrage.
    SpellsCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has cast no spell from hand this turn (CR 601); other zones don't
    /// count. Prairie Dog.
    NoSpellCastFromHandThisTurn { who: PlayerRef },
    /// The just-cast spell is the turn's first noncreature spell by anyone
    /// (`GameState.noncreature_spells_cast_this_turn == 1`, already counting it).
    /// Nullstone Gargoyle; pair with a noncreature-spell event filter.
    FirstNoncreatureSpellThisTurn,
    /// `who` has cast exactly `count` spells this turn, counting the current one
    /// ("their second spell each turn" — Ledger Shredder; pair with
    /// `PlayerRef::Triggerer` + `EventScope::AnyPlayer`).
    SpellsCastThisTurnEquals { who: PlayerRef, count: Value },
    /// No spells were cast by any player during the previous turn — the
    /// classic Innistrad day-side werewolf transform check. Backed by
    /// `GameState.spells_cast_last_turn == 0`.
    NoSpellsCastLastTurn,
    /// Two or more spells were cast during the previous turn — the night-side
    /// werewolf "transform back" check. Backed by
    /// `GameState.spells_cast_last_turn >= 2`.
    TwoOrMoreSpellsCastLastTurn,
    /// At least `at_least` of `who`'s creatures died this turn
    /// (`Player.creatures_died_this_turn`). Essenceknit Scholar.
    CreaturesDiedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// A creature matching `filter` died this turn, under any controller
    /// ("if a non-Zombie creature died this turn" — Undead Sprinter). Read
    /// against the death-time definitions in `creature_deaths_this_turn`.
    CreatureDiedThisTurnMatching { filter: SelectionRequirement },
    /// CR 122 — at least `at_least` distinct counter kinds among `who`'s
    /// creatures (Hundred-Battle Veteran).
    DistinctCounterKindsAmongCreaturesAtLeast { who: PlayerRef, at_least: u32 },
    /// `who` has sacrificed at least `at_least` permanents this turn. Backed
    /// by `Player.permanents_sacrificed_this_turn`. Used by "if you
    /// sacrificed a permanent this turn" payoffs (Sawblade Skinripper).
    PermanentsSacrificedThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has sacrificed at least one *artifact* this turn. Backed by
    /// `Player.artifacts_sacrificed_this_turn`. Used by "if you've sacrificed an
    /// artifact this turn" riders (Suspicious Detonation, Furtive Courier).
    SacrificedArtifactThisTurn { who: PlayerRef },
    /// `who` has sacrificed a Food this turn (Elanor Gardner). Backed by
    /// `Player.foods_sacrificed_this_turn`.
    SacrificedFoodThisTurn { who: PlayerRef },
    /// CR 700.4 Morbid — at least `at_least` creatures died this turn under any
    /// player's control (summed over seats). Tragic Slip.
    CreaturesDiedThisTurnTotalAtLeast { at_least: Value },
    /// `who` has exiled at least `at_least` cards this turn
    /// (`Player.cards_exiled_this_turn`). Ennis the Debate Moderator.
    CardsExiledThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// "…if there are N or more cards in exile" (Ketramose's attack / block
    /// gate). Counts every owner's exiled cards.
    CardsInExileAtLeast(u32),
    /// `who` has cast at least `at_least` instants or sorceries this turn
    /// (`Player.instants_or_sorceries_cast_this_turn`). Potioner's Trove.
    InstantsOrSorceriesCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has cast at least `at_least` creature spells on the current
    /// turn. Backed by `Player.creatures_cast_this_turn`. Reserved for
    /// future "if you've cast a creature spell this turn, …" payoffs.
    CreaturesCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// `who` has cast at least `at_least` noncreature spells this turn (Eshki
    /// Dragonclaw).
    NoncreatureSpellsCastThisTurnAtLeast { who: PlayerRef, at_least: Value },
    /// The spell at `Selector::TriggerSource` has a chosen target matching
    /// `filter` (topmost matching `StackItem::Spell`). Repartee: with
    /// `cast_is_instant_or_sorcery()`, "a spell that targets a creature".
    CastSpellTargetsMatch(SelectionRequirement),
    /// The just-cast spell has exactly one target in total and it matches the
    /// filter ("…that targets only a single creature you control" — Leyline
    /// of Resonance). Counts slot 0 plus every additional target.
    CastSpellTargetsOnlyOneMatching(SelectionRequirement),
    /// The resolving instant or sorcery targets only a single object, and it is
    /// this trigger's target (Imodane). Read off the resolution snapshot, since
    /// the card is in no zone while it resolves.
    ResolvingInstantOrSorceryTargetsOnlyTriggerTarget,
    /// The just-cast spell (topmost matching `StackItem::Spell`) matches
    /// `filter`. Sprite Dragon.
    CastSpellMatches(SelectionRequirement),
    /// The just-cast spell matches `filter` and is its caster's first such spell
    /// this turn (Alania, Divergent Storm).
    CastSpellFirstMatchingThisTurn(SelectionRequirement),
    /// On an `AuraAttachedToAny` trigger: the Aura (`ctx.trigger_source`) is
    /// attached to a nonland permanent an opponent controls whose mana value is
    /// at most the Aura's (Eriette, the Beguiler).
    AuraHostIsCheaperOpponentPermanent,
    /// True if the just-cast spell (located via `ctx.trigger_source`) was cast
    /// as an Adventure (CR 716 — its `adventuring` flag is set). Powers
    /// Chancellor of Tales' "whenever you cast an Adventure spell".
    CastSpellIsAdventure,
    /// The trigger object (just-cast spell or played land) shares a card type
    /// with the card this source exiled. Innistrad's Cemetery cycle.
    SharesCardTypeWithExiledBySource,
    /// The just-cast spell shares a colour or mana value with the card exiled
    /// with the trigger's source (Thought Prison's imprint).
    CastSharesColorOrManaValueWithExiledBySource,
    /// The activation's chosen X equals the mana value of the card exiled with
    /// the source (Soul Foundry's "{X}, {T}" — X is that card's mana value).
    ExiledWithSourceManaValueIsX,
    /// The just-cast spell was kicked (its stack instance's flag), for cast
    /// triggers (Scourge of the Skyclaves).
    CastSpellWasKicked,
    /// `CastSpellWasKicked` for one of a "kicker {A} and/or {B}" spell's
    /// kickers, by index (Wastescape Battlemage's two cast triggers).
    CastSpellWasKickedWith(u8),
    /// "If you control a commander" (Akroma's Will, Jeska's Will) — true
    /// when any battlefield permanent the effect's controller controls is
    /// one of their designated commanders (`Player.commanders`).
    YouControlACommander,
    /// CR 601.2 — the card's `as_cast_condition` held as it was cast: the
    /// resolving spell's stamp, or (an ETB trigger) its permanent's.
    CastConditionHeld,
    /// "If you've cast a spell from a graveyard or activated an ability of a
    /// card in a graveyard this turn" (Laboratory Drudge). Reads
    /// `Player.used_graveyard_this_turn`.
    UsedGraveyardThisTurn { who: PlayerRef },
    /// "Unless they control a commander" (Crimson Honor Guard) — true when
    /// every player `who` names controls a permanent that is any player's
    /// commander (CR 903.3).
    PlayerControlsACommander { who: PlayerRef },
    /// The just-cast spell (`ctx.trigger_source`) has `{X}` in its mana cost
    /// (`ManaCost::has_x`). Quandrix payoffs (Geometer's Arthropod).
    CastSpellHasX,
    /// CR 602.1a / 107.3 — the activation that fired this `AbilityActivated`
    /// trigger has `{X}` in its cost (Unbound Flourishing).
    ActivationCostHasX,
    /// The just-cast spell's total mana spent (`StackItem::Spell.mana_spent`,
    /// carried onto the trigger) is at least `at_least`. Opus and Increment
    /// payoffs.
    CastSpellManaSpentAtLeast(u32),
    /// CR 702.137 (Adamant) — at least `at_least` mana of `color` was spent
    /// casting the resolving spell. Reads the per-color payment breakdown
    /// stamped at cast time (`EffectContext.mana_spent_by_color`).
    ManaSpentOfColorAtLeast { color: crate::mana::Color, at_least: u32 },
    /// Adamant for colorless — "if at least N colorless mana was spent to cast
    /// this spell" (Desecrate Reality): the resolving spell's total spend less
    /// its colored breakdown.
    ColorlessManaSpentAtLeast(u32),
    /// "If [color] mana was spent to cast this", read from the source's own cast
    /// provenance (`CardInstance.cast_mana_spent_by_color`) for ETB triggers
    /// (Gruul Scrapper).
    SourceCastWithColorSpent { color: crate::mana::Color, at_least: u32 },
    /// CR 702.137 Adamant with no color named (Henge Walker), from the source's
    /// cast provenance. Not five `SourceCastWithColorSpent` arms: those colors
    /// would enter the card's CR 903.4 identity.
    SourceCastWithOneColorSpent { at_least: u32 },
    /// True if no *colored* mana was spent casting the just-cast spell
    /// (`ctx.trigger_source`) — Void Mirror's counter gate. Free casts
    /// (suspend, cascade) spend no mana, so they match.
    CastSpellNoColoredManaSpent,
    /// True if the just-cast spell (`ctx.trigger_source`) is one of the source
    /// permanent's chosen color (`CardInstance.chosen_color` on `ctx.source`).
    /// Diamond Mare's "whenever you cast a spell of the chosen color" payoff.
    CastSpellSharesChosenColorOfSource,
    /// Whether `{C}` was spent on the just-cast spell: `true` = Drowner of Truth,
    /// `false` = Wumpus Aberration. Total spent minus the colored breakdown, so
    /// `{C}` paid for generic counts; a free cast spent none.
    CastSpellColorlessManaSpent { spent: bool },
    /// The just-cast spell's owner (CR 108.3) is not `ctx.controller` — "whenever
    /// you cast a spell you don't own" (Nita, Forum Conciliator).
    CastSpellNotOwnedByYou,
    /// The just-cast spell's owner isn't its caster (judged off the stack item,
    /// unlike [`Predicate::CastSpellNotOwnedByYou`]) — Gonti, Night Minister.
    CastSpellNotOwnedByCaster,
    /// True if the just-cast spell (via `ctx.trigger_source`) was cast from
    /// exile — reads the `StackItem::Spell.card.cast_from_exile` flag.
    /// Nassari, Dean of Expression's "whenever you cast a spell from exile".
    CastSpellFromExile,
    /// The just-cast spell was cast from its caster's hand (stack flag). Jodah,
    /// the Unifier. `CastFromHand` is the resolving spell's own flag.
    CastSpellFromHand,
    /// `CastSpellFromExile`, and it is the caster's first spell cast from
    /// exile this turn (`Player.spells_cast_from_exile_this_turn == 1`) —
    /// Wild-Magic Sorcerer.
    FirstSpellCastFromExileThisTurn,
    /// `who` played a card from exile this turn — cast a spell or played a
    /// land (CR 305.1 / 601.2; Visions of Phyrexia's "if you didn't play a
    /// card from exile this turn").
    PlayedCardFromExileThisTurn(PlayerRef),
    /// Two of the cards `what` resolves to share all their card types
    /// (identical card-type sets) — Demonic Covenant's "if two cards that
    /// share all their card types were milled this way".
    TwoShareAllCardTypes(Selector),
    /// The just-cast spell was cast from its owner's library. Melek, Izzet
    /// Paragon.
    CastSpellFromLibrary,
    /// The Fourth Doctor's "when you do": the triggering spell or land was played
    /// under this source's top-of-library grant
    /// (`PlayerCold::library_top_grant_used`).
    UsedSourcesLibraryTopGrant,
    /// True if the just-cast spell (via `ctx.trigger_source`) was cast from a
    /// graveyard — reads `StackItem::Spell.card.cast_from_graveyard`. River
    /// Kelpie's "whenever a player casts a spell from a graveyard".
    CastSpellFromGraveyard,
    /// The just-cast spell is the controller's first instant or sorcery cast
    /// from a graveyard this turn (Sevinne, the Chronoclasm).
    FirstInstantOrSorceryCastFromGraveyardThisTurn,
    /// The Ring has tempted `who` at least `n` times this game (CR 701.54 —
    /// Frodo, Adventurous Hobbit's "two or more times").
    RingTemptedAtLeast { who: PlayerRef, n: u32 },
    /// The source has had a counter put on it this turn
    /// (`permanents_gained_counter_this_turn`, cleared at cleanup). Fractal
    /// Tender's end-step rider.
    SourceGainedCounterThisTurn,
    /// The source has at least `n` `counter` counters (Charitable Levy).
    SourceHasCountersAtLeast { counter: crate::card::CounterType, n: u32 },
    /// `ctx.source` has a `keyword` counter on it (CR 122.1b) — "you may
    /// remove a deathtouch counter from it. When you do" (Kappa Tech-Wrecker).
    SourceHasKeywordCounter(crate::card::Keyword),
    /// CR 717.2 — the source Class is at exactly level `n`. Gates its "Level N"
    /// ability (`SourceClassLevelIs(N-1)`) and its "becomes level N" trigger
    /// (`ClassLevelReached` filtered on `SourceClassLevelIs(N)`).
    SourceClassLevelIs(u8),
    /// CR 717.2 — true when `ctx.source` (a Class enchantment) is at level `n`
    /// or higher. Gates a Class's level-`n` triggered abilities so they only
    /// fire once the Class has reached that level.
    SourceClassLevelAtLeast(u8),
    /// The just-cast spell's mana spent is strictly greater than the source's
    /// power or toughness — SOS Increment.
    IncrementSatisfied,
    /// `who`'s `zone` holds at least `at_least` cards named like the resolving
    /// spell (`EffectContext.source_name`; `False` with no source). Dragon's
    /// Approach.
    SameNamedInZoneAtLeast { who: PlayerRef, zone: Zone, at_least: Value },
    /// The resolving spell was cast from its caster's graveyard (Flashback,
    /// Jump-Start, …; `EffectContext.cast_from_graveyard`). Increasing Vengeance.
    CastFromGraveyard,
    /// The resolving spell or ability is controlled by an opponent of the
    /// evaluating controller (`GameState.resolution_causer`). Sacred Ground.
    ///
    /// ⚠ As a trigger filter this is only sound on an event whose dispatch
    /// replays the causer: `resolution_causer` is cleared when the resolution
    /// ends, before triggers dispatch. Only `pending_permanent_deaths` carries it
    /// across (`PermanentDied`). A discard rider wants
    /// `EventKind::OpponentCausedYouToDiscard`, which decides at emission (Pure
    /// Intentions).
    #[serde(alias = "DiscardCausedByOpponent")]
    CausedByOpponentSpellOrAbility,
    /// The resolving spell was cast from its caster's hand. Not the inverse of
    /// `CastFromGraveyard` (exile / library / command casts are neither). Quandrix,
    /// the Proof. Triggers and activated abilities default `cast_from_hand` to
    /// `true`, so this is `True` outside spell resolution.
    CastFromHand,
    /// The two player refs resolve to the same seat — "whenever enchanted
    /// player is dealt damage" is `SamePlayer(TriggerEventPlayer,
    /// EnchantedPlayer)` (Grievous Wound).
    SamePlayer(PlayerRef, PlayerRef),
    /// No extra combat has begun this turn — so an attack rider's extra combat
    /// doesn't loop (Genji Glove).
    IsFirstCombatPhaseThisTurn,
    /// The turn's current combat phase is its `n`th (1-based) — pins a
    /// delayed trigger to the combat it was created in (CR 506.1).
    CombatPhaseNumberIs(u32),
    /// True while the current turn is in (or has only ever reached) its first
    /// end step — i.e. no extra end step has begun yet. Gates "if it's the
    /// first end step of the turn, …" riders (Y'shtola Rhul).
    IsFirstEndStepThisTurn,
    /// True while the current turn is in its first upkeep step — gates
    /// Paradox Haze's "first upkeep step of your turn" trigger so the extra
    /// upkeep it grants doesn't loop.
    IsFirstUpkeepThisTurn,
    /// You put a counter on a creature this turn (CR 122.6 — Lasting
    /// Tarfire's intervening "if you put a counter on a creature this turn").
    CounterPutOnCreatureThisTurn,
    /// The trigger's subject became tapped for the first time this turn
    /// (`CardInstance.tapped_this_turn` without `tapped_again_this_turn`) —
    /// Captain America, Living Legend.
    TriggerSourceFirstTappedThisTurn,
    /// An opponent of the controller has cast a spell this turn (Captain
    /// Mar-Vell, Space-Born).
    AnOpponentCastASpellThisTurn,
    /// A player attacked the controller during that player's last turn
    /// (Avenge — `attacked_players_this_turn`, rolled over as their turn
    /// begins).
    APlayerAttackedYouLastTurn,
    /// `.0` attacked the controller during the current turn (Faramir's "if
    /// they didn't attack you that turn", read at that player's end step).
    PlayerAttackedYouThisTurn(PlayerRef),
    /// CR 506.2 — one or more of the controller's opponents (or their
    /// planeswalkers or battles they protect) is being attacked this combat,
    /// by anyone (Party Dude's "one or more of your opponents are attacked").
    AnOpponentIsAttacked,
    /// The resolving spell was kicked (CR 702.32; `EffectContext.kicked`). Tear
    /// Asunder. `false` outside a spell.
    SpellWasKicked,
    /// CR 702.33b — the resolving spell was kicked with option `n` of its
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
    /// The entering permanent arrived from a graveyard this turn or was cast from
    /// one (`!cast_from_hand`; exile casts over-trigger). Breathless Knight.
    TriggerSourceEnteredFromGraveyard,
    /// The entering permanent bound to `ctx.trigger_source` entered because its
    /// spell was cast (reads `CardInstance.entered_by_cast`). Gates "whenever a
    /// creature you control enters, if you cast it, …" (The Sibsig Ceremony).
    TriggerSourceEnteredByCast,
    /// CR 702.85 Heroic — the just-cast spell targets the trigger's source
    /// (read off the stack spell's `target` / `additional_targets`).
    CastSpellTargetsSource,
    /// CR 702.176 — true iff the Bargain additional cost was paid (an
    /// artifact, enchantment, or token sacrificed) at cast time. Reads
    /// `EffectContext.bargained`, stamped from `CardInstance.bargained`.
    SpellWasBargained,
    /// CR 702.187 — this spell was cast for its Mayhem cost
    /// (`EffectContext.cast_via_mayhem`). Sandman's Quicksand.
    SpellWasMayhem,
    /// CR 702.35 — "if its madness cost was paid". Reads
    /// `EffectContext.cast_via_madness`, stamped from
    /// `CardInstance.cast_via_madness`.
    SpellWasMadness,
    /// "If this spell was cast from exile" (Lifestream's Blessing, the
    /// Ultimate Magic cycle) — the resolving spell's own cast zone.
    SpellWasCastFromExile,
    /// "If you do" after a resolve-now "you may cast" (`CastWithoutPayingImmediate`
    /// and its siblings): a spell was cast that way earlier in this resolution
    /// (Soundwave, Sonic Spy).
    CastImmediatelyThisResolution,
    /// CR 701.67 — this spell's optional waterbend cost was paid
    /// (`EffectContext.cast_via_waterbend`). Katara, Seeking Revenge.
    SpellWasWaterbend,
    /// CR 701.59 — this spell's collect-evidence cost was paid
    /// (`EffectContext.cast_collected_evidence`). Behind the Mask.
    SpellCollectedEvidence,
    /// CR 702.165 — the source was cast with its gift promised
    /// (`CardInstance.gift_promised`). Scrapshooter.
    SourceGiftPromised,
    /// "Do this only once each turn": the source already did it this turn
    /// (`Effect::MarkDoneThisTurn`). Gates the trigger; see
    /// `shortcut::may_once_each_turn`.
    SourceDoneThisTurn,
    /// CR 603.3d — the source's trigger `index` already used its once-each-turn
    /// budget, so two printed triggers can share one (Bill Potts).
    SourceTriggerUsedThisTurn(u8),
    /// True if the most recently discarded card this resolution had mana value
    /// ≤ `n`. Reads `GameState.last_discarded_mana_value` (Hollow Marauder's
    /// "draw unless they discarded a card with mana value 4 or greater").
    LastDiscardedManaValueAtMost(u32),
    /// CR 716 — the controller owns an exiled card on an Adventure (Howling
    /// Galefang).
    OwnExiledAdventureCard,
    /// Some opponent controls more lands than `ctx.controller` (Gift of Estates,
    /// Land Tax).
    OpponentControlsMoreLandsThanYou,
    /// An opponent controls at least `n` more lands than you (Isolated
    /// Watchtower's "at least two more").
    OpponentControlsAtLeastMoreLands(u32),
    /// True if any opponent of `ctx.controller` has a strictly higher life
    /// total. Linvala, the Preserver's first ETB ("if an opponent has more
    /// life than you, you gain 5 life").
    AnOpponentHasMoreLife,
    /// True if any opponent of `ctx.controller` controls strictly more
    /// creatures. Linvala, the Preserver's second ETB ("if an opponent
    /// controls more creatures than you, create a 3/3 Angel").
    AnOpponentControlsMoreCreatures,
    /// A single opponent (not the table summed, CR 102.2) controls at least `n`
    /// `filter` permanents — Razorlash Transmogrant's "four or more nonbasic
    /// lands".
    AnOpponentControlsAtLeast { filter: crate::card::SelectionRequirement, n: u32 },
    /// True if any opponent of `ctx.controller` has strictly more cards in
    /// hand. Beza, the Bounding Spring's "draw a card if an opponent has more
    /// cards in hand than you".
    AnOpponentHasMoreCardsInHand,
    /// Coven: `who` controls three or more creatures with different powers
    /// (Sigarda, Champion of Light).
    CovenActive { who: PlayerRef },
    /// CR 702.166 — Corrupted (Phyrexia: All Will Be One ability word): an
    /// opponent of `who` has three or more poison counters. Gates Corrupted
    /// triggers / "as long as" statics.
    CorruptedActive { who: PlayerRef },
    /// Churning Reservoir — an oil counter was removed from `who`'s permanent
    /// this turn, or an oil-countered permanent of theirs hit a graveyard.
    OilActivityThisTurn { who: PlayerRef },
    /// `who` controls a creature with the greatest power on the battlefield (ties
    /// count); false with no creature. Summon: Fenrir III.
    ControlsGreatestPowerCreature { who: PlayerRef },
    /// The toughness twin of `ControlsGreatestPowerCreature` (Abzan
    /// Beastmaster's "the creature with the greatest toughness or tied").
    ControlsGreatestToughnessCreature { who: PlayerRef },
    /// Exactly one creature is attacking — CR 702.83a "attacks alone" (Exalted,
    /// with an `Attacks / YourControl` trigger). `false` outside combat.
    AttackingAlone,
    /// CR 506.2 — the trigger source attacks a player no other creature is
    /// attacking (Jin Sakai). Unlike `AttackingAlone`, attacks on other players
    /// don't count — a pod-only distinction.
    TriggerSourceAttacksItsPlayerAlone,
    /// At least `n` creatures are attacking — Battalion is `n == 3`. `false`
    /// outside combat.
    AttackingWithAtLeast(u32),
    /// Pack tactics (OTJ) — `who`'s attacking creatures this combat have total
    /// computed power at least `at_least`. Battle Cry Goblin.
    AttackedWithTotalPowerAtLeast { who: PlayerRef, at_least: u32 },
    /// `who` declared at least `at_least` attackers this combat; `false` outside
    /// combat. Argent Dais (with `ActivePlayer`).
    AttackedWithCountAtLeast { who: PlayerRef, at_least: u32 },
    /// CR 506.2 — a player being attacked directly this combat has at least
    /// `at_least` poison. Norn's Decree.
    AnAttackedPlayerHasPoisonAtLeast { at_least: u32 },
    /// CR 506.2 / 508.1 — `who` declared at least `at_least` attackers this combat
    /// at `defender` (and, with `include_planeswalkers`, at planeswalkers
    /// `defender` controls). The defender-side refinement of
    /// `AttackedWithCountAtLeast`, which counts the whole declaration.
    ///
    /// ⚠ Identical in a duel, different in a pod: a swing at a third player is
    /// not an attack on you.
    ///
    /// `include_planeswalkers` follows the printed text: Trouble in Pairs says
    /// "attacks you" (false, CR 506.2); Mangara, the Diplomat says "you and/or
    /// planeswalkers you control" (true). Attacks on Battles count for neither.
    ///
    /// ⚠ Reads `GameState.attacking`, complete only once the whole declaration
    /// is in (see the defender-side walk in `combat.rs`): on a
    /// `ControllerAttackedByOpponent` trigger put it in `Effect::If`, not the
    /// `EventSpec` filter.
    AttackedDefenderWithCountAtLeast {
        who: PlayerRef,
        defender: PlayerRef,
        at_least: u32,
        include_planeswalkers: bool,
    },
    /// `who` attacked with a `filter` creature this combat; `false` outside
    /// combat. Teo, Spirited Glider.
    AttackedWithCreatureMatching { who: PlayerRef, filter: SelectionRequirement },
    /// `player` is attacked (directly) by a `filter` creature the evaluating
    /// controller controls (Zurzoth, Chaos Rider).
    PlayerAttackedByMatching { player: PlayerRef, filter: SelectionRequirement },
    /// Some opponent's top card shares a card type with `what` (Gandalf,
    /// Westward Voyager).
    AnOpponentsTopCardSharesCardTypeWith(Selector),
    /// CR 700.13 — `who` has committed a crime this turn. Backed by
    /// `Player.committed_crime_this_turn`. Powers "as long as / if you've
    /// committed a crime this turn" riders (Nimble Brigand's evasion).
    CommittedCrimeThisTurn { who: PlayerRef },
    /// CR 708 — a permanent entered the battlefield face down under `who`'s
    /// control, or they turned a permanent face up, this turn. Backed by
    /// `Player.face_down_activity_this_turn` (Oblivious Bookworm).
    FaceDownActivityThisTurn { who: PlayerRef },
    /// CR 709.5 — `who` has at least `count` unlocked doors among their Rooms
    /// (`CardInstance.unlocked_doors` bitmask). Rampaging Soulrager.
    UnlockedDoorsControlledAtLeast { who: PlayerRef, count: u32 },
    /// CR 709.5 — `who` controls at least `count` unlocked doors with
    /// **distinct names** (Promising Stairs' "eight or more different names
    /// among unlocked doors of Rooms you control" win condition).
    DistinctUnlockedDoorNamesAtLeast { who: PlayerRef, count: u32 },
    /// CR 700.12 — `who` controls an **outlaw**: a creature that is an
    /// Assassin, Mercenary, Pirate, Rogue, or Warlock. Powers "as long as you
    /// control an outlaw" / "if you control an outlaw" riders (Take the Fall).
    ControlsOutlaw { who: PlayerRef },
    /// Delirium: `who`'s graveyard has 4+ distinct card types (Unholy Heat).
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
    /// Another creature entered under `who`'s control this turn — the
    /// self-excluding sibling of `CreatureEnteredThisTurn` (Bellowing Elk).
    AnotherCreatureEnteredThisTurn { who: PlayerRef },
    /// Guardian Project: no other creature you control and no creature card in
    /// your graveyard shares the trigger subject's name.
    TriggerSourceNameUniqueAmongYourCreatures,
    /// Celebration (WOE): two or more nonland permanents entered under `who`
    /// this turn (`Player.nonland_permanents_entered_this_turn`). Armory Mice.
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
    /// "If a Food entered the battlefield under `who`'s control this turn"
    /// (Motivated Pony). Reads `Player.foods_entered_this_turn`.
    FoodEnteredThisTurn { who: PlayerRef },
    /// CR 309.7 — `who` has completed the dungeon named `name` this game
    /// (Acererak the Archlich's "if you haven't completed Tomb of
    /// Annihilation"). Reads `PlayerCold::completed_dungeons`.
    CompletedDungeon { who: PlayerRef, name: String },
    /// `who` has cast a spell named `name` this turn (Sift Through Sands'
    /// "a spell named Peer Through Depths"). Reads
    /// `Player::spell_names_cast_this_turn`.
    CastSpellNamedThisTurn { who: PlayerRef, name: String },
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
    /// "Until your next turn" (CR 611.2b), mapped beside `UntilYourNextUntap`.
    /// Before 2026-09-07 it ended at any player's next turn (Liliana's -2/-1
    /// expired before the opponent's combat).
    UntilNextTurn,
    /// CR 611.2c — "for as long as this permanent remains tapped" (Thran
    /// Weaponry). The affected set is locked in at resolution; the SBA sweep
    /// drops the effect once the source untaps or leaves.
    WhileSourceTapped,
    /// CR 611.2c — "for as long as this Equipment/Aura remains attached to it"
    /// (Assimilation Aegis). The SBA sweep drops the effect once the source
    /// stops being attached or leaves the battlefield.
    WhileSourceAttached,
    /// CR 611.2c — "for as long as this creature remains on the battlefield"
    /// (Tishana's Tidebinder), for an effect a resolution installs; the SBA sweep
    /// drops it once the source is gone. Statics need no sweep.
    WhileSourceOnBattlefield,
    /// CR 611.2b — "for as long as you control [this]": also ends when
    /// another player gains control of the source (Aegis Angel).
    WhileYouControlSource,
    /// CR 611.2b — "until the end of your next turn" (Treasure Nabber): lasts
    /// through the controller's next turn — the one after this if it's
    /// already theirs — and ends in that turn's cleanup step.
    UntilEndOfYourNextTurn,
    /// Indefinite (for effects like "gain control" without a clause).
    Permanent,
    /// "Until this card is cast from exile" (Emrakul, the Exigent Doom): a grant
    /// `finalize_cast` strips when the source is next cast from exile (CR 611.2b).
    UntilSourceCast,
    /// CR 708.8 — "until this creature is turned face down" (Vesuvan
    /// Shapeshifter's copy): ends as `Effect::TurnFaceDown` turns it, so the
    /// card it stashes is its own.
    UntilTurnedFaceDown,
    /// CR 611.2c — "for as long as it has a [kind] counter on it": the SBA
    /// sweep drops the effect once an affected permanent has none (Aven
    /// Mimeomancer's feather, Xolatoyac's flood), whatever its source does.
    WhileHasCounter(crate::card::CounterType),
    /// The keyword-counter sibling: "for as long as that creature has a
    /// shadow counter on it" (Minas Morgul). CR 122.1b's kinds only.
    WhileHasKeywordCounter(KeywordCounterKind),
    /// CR 611.2c — "for as long as it remains tapped" read of the AFFECTED
    /// permanent (Braided Net's lock), not the source (`WhileSourceTapped`).
    WhileTapped,
}

/// CR 122.1b — the keyword counters, as a `Copy` tag a `Duration` can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeywordCounterKind {
    Flying,
    FirstStrike,
    DoubleStrike,
    Deathtouch,
    Haste,
    Hexproof,
    Indestructible,
    Lifelink,
    Menace,
    Reach,
    Shadow,
    Trample,
    Vigilance,
}

impl KeywordCounterKind {
    pub fn keyword(self) -> crate::card::Keyword {
        use crate::card::Keyword as K;
        match self {
            Self::Flying => K::Flying,
            Self::FirstStrike => K::FirstStrike,
            Self::DoubleStrike => K::DoubleStrike,
            Self::Deathtouch => K::Deathtouch,
            Self::Haste => K::Haste,
            Self::Hexproof => K::Hexproof,
            Self::Indestructible => K::Indestructible,
            Self::Lifelink => K::Lifelink,
            Self::Menace => K::Menace,
            Self::Reach => K::Reach,
            Self::Shadow => K::Shadow,
            Self::Trample => K::Trample,
            Self::Vigilance => K::Vigilance,
        }
    }
}

// ── Library positions, scry modes, mana ──────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryPosition {
    Top,
    Bottom,
    Shuffled,
    /// The card's owner chooses top or bottom (Run Behind), asked via
    /// `Decision::OptionalTrigger` at `place_card_in_dest` time; yes = top.
    /// AutoDecider's default puts it on the bottom.
    OwnerChoice,
    /// Deem Inferior: the owner chooses second from the top (`FromTop(1)`) or the
    /// bottom via `Decision::OptionalTrigger`; AutoDecider picks bottom.
    SecondFromTopOrBottom,
    /// Nth from the top of the owner's library (Approach of the Second Sun's
    /// seventh). With fewer than N cards it goes on the bottom (CR 401.7).
    /// `FromTop(0)` is `Top`.
    FromTop(usize),
    /// "Just beneath the top X cards" — `FromTop(X)` with the resolving
    /// spell's X (Unexpectedly Absent). Flattened to `FromTop` while the
    /// destination is resolved against the effect's context.
    BeneathTopX,
}

/// Where a `RevealUntilFind`'s non-matching cards go. `Graveyard` is the
/// historical default; SOS cards that print "the rest on the bottom in a
/// random order" (Geometer's Arthropod) use `BottomRandom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RevealMissDest {
    /// Misses go to the controller's graveyard (legacy / Spoils-style).
    #[default]
    Graveyard,
    /// Misses go on the bottom of the controller's library in revealed order —
    /// no RNG hook here, and nothing reads the bottom before the next shuffle.
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
    /// CR 702.170 — exile face up as plotted, castable free as a sorcery on a
    /// later turn; no plot cost paid (Kellan Joins Up).
    ExilePlotted,
    /// Exile, stamping `exiled_with = ctx.source` so the source can reach the
    /// card later (`Selector::CardExiledWithSource`, Kaho's free cast).
    ExileWithSourceStamp,
    /// Battlefield under `controller`, optionally tapped.
    Battlefield { controller: PlayerRef, tapped: bool },
    /// CR 407.4 — the moved card's owner's ante zone.
    Ante,
    /// CR 408 — the moved card's owner's command zone ("return it to the
    /// command zone", Hellkite Courser). A direct move, not the optional CR
    /// 903.9 redirect.
    Command,
    /// "If it's a [filter] card, put it [`then`]; otherwise [`else_`]", read off
    /// the card as it is placed (Nazahn's Hammer).
    IfCard { filter: Box<SelectionRequirement>, then: Box<ZoneDest>, else_: Box<ZoneDest> },
    /// CR 303.4f / 301.5c — onto the battlefield attached to `host` (Unfinished
    /// Business); one that can't legally attach stays (CR 303.4i).
    BattlefieldAttached { host: Box<Selector> },
}

/// Where a countered spell goes after being lifted off the stack. The
/// default (graveyard) matches CR 701.6a; Memory Lapse routes to the
/// owner's library top, Spell Crumple routes to exile, etc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounteredSpellZone {
    /// Top of the spell-owner's library (Memory Lapse).
    OwnerLibraryTop,
    /// Bottom of the spell-owner's library.
    OwnerLibraryBottom,
    /// Owner's choice of top or bottom of their library (Subtlety).
    OwnerLibraryTopOrBottom,
    /// Owner's hand (Remand).
    OwnerHand,
    /// Second from the top of the owner's library (Commit).
    OwnerLibrarySecondFromTop,
    /// Exile (Spell Crumple).
    Exile,
    /// CR 702.170 — exile and mark plotted, so its owner may cast it for free
    /// as a sorcery on a later turn (Aven Interrupter).
    ExilePlotted,
    /// Exile stamped `exiled_with` = the countering source, so the source can
    /// name what it took later (Shell of the Last Kappa).
    ExileWithSource,
    /// CR 702.62a — exile with `n` time counters; a card without suspend gains
    /// it (Delay).
    ExileSuspended(u32),
    /// "…put that card onto the battlefield under your control instead"
    /// (Desertion). Cards that fail the filter fall back to the graveyard.
    CountererBattlefieldIfMatching(Box<SelectionRequirement>),
    /// Thranduil's Decree: a countered card matching the filter is exiled with a
    /// free may-cast for the countering player (CR 601.2 / 118.9); the rest go to
    /// the graveyard.
    ExileCountererMayCastFreeIfMatching(Box<SelectionRequirement>),
}

/// What mana to add to a pool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManaPayload {
    /// One mana of a color of the card imprinted on the source (`exiled_with`);
    /// nothing if none or colorless. Chrome Mox.
    ImprintedCardColor,
    /// Add each listed color once.
    Colors(Vec<Color>),
    /// Add `amount` colorless mana.
    Colorless(Value),
    /// `amount` mana of one fixed color (Topiary Lecturer's power-scaled {G}).
    OfColor(Color, Value),
    /// Add `amount` mana of any one color (player chooses).
    AnyOneColor(Value),
    /// Add `amount` mana of any colors (player chooses each).
    AnyColors(Value),
    /// `n` mana of different colors (CR 106.1 — Component Pouch, Guild Globe):
    /// [`Self::AnyColors`] with no color picked twice.
    DifferentColors(Value),
    /// `amount` mana, each pip chosen from the given colors (Culling Ritual's "{B}
    /// or {G}"). Falls back to the first color when the decider can't choose.
    OfColors(Vec<Color>, Value),
    /// CR 903.4 — one mana of any color in the controller's commanders' combined
    /// identity (Command Tower, Arcane Signet).
    ///
    /// With no commander it produces any color, as it did before Commander
    /// existed, so cube and constructed games (and their traces) are unchanged.
    AnyColorInCommanderIdentity,
    /// "Either of the circled colors" (Cryptic Spires). There is no deck-building
    /// choice, so it reads the commander identity: exact for one or two colors,
    /// its first two (WUBRG order) otherwise; white and blue with no commander.
    CircledColors,
    /// One mana of any color an opponent's basic-typed land could produce
    /// (Fellwar Stone); colorless when there is none, so it never no-ops.
    AnyColorOpponentCouldProduce,
    /// CR 106.7 — one mana of a color a land you control could produce,
    /// intrinsic basic-type colors included (CR 305.6); none, no mana. Harvester
    /// Druid.
    AnyColorYouCouldProduce,
    /// CR 106.7 — [`Self::AnyColorYouCouldProduce`] over your *basic* lands
    /// only (Star Compass).
    AnyColorABasicLandYouControlCouldProduce,
    /// Gond Gate — "one mana of any color that a Gate you control could
    /// produce": the colors the Gates' own mana abilities (and chosen colors)
    /// make.
    AnyColorAGateYouControlCouldProduce,
    /// CR 106.7 — `n` mana of one type (colors or {C}) a land you control could
    /// produce, read off the board (Reflecting Pool). Nothing when no land could
    /// produce any.
    AnyTypeALandYouControlCouldProduce(Value),
    /// "One mana of any type that a Gate you control could produce" (Plaza
    /// of Harmony): [`Self::AnyTypeALandYouControlCouldProduce`] over Gates.
    AnyTypeAGateYouControlCouldProduce,
    /// Add one mana of any type the *trigger's subject* land produced
    /// (Extraplanar Lens). Falls back to colorless if it produces nothing.
    AnyTypeTriggerSourceProduces,
    /// Add one mana of any type a land sacrificed to pay this ability's cost
    /// could produce (Squandered Resources). The controller picks among the
    /// sacrificed lands' basic types; a typeless land yields {C}.
    AnyTypeSacrificedLandProduces,
    /// One mana of any color among the cards exiled with this source (Pit of
    /// Offerings); nothing when that union is empty, as printed.
    AnyColorAmongExiledWithSource,
    /// Add one mana of any color among legendary creatures and planeswalkers
    /// you control (Mox Amber). The legal-color set is the union of those
    /// permanents' colors; produces nothing when empty.
    AnyColorAmongLegendaries,
    /// Add one mana of any color among legendary creature cards in your
    /// graveyard (The Grey Havens) — their printed colors; nothing when there
    /// are none.
    AnyColorAmongLegendaryCreatureCardsInYourGraveyard,
    /// "Choose a color of a card in your graveyard. Add one mana of that
    /// color" (Corrupted Grafstone) — any card's printed colors; nothing when
    /// the graveyard holds only colorless cards.
    AnyColorAmongCardsInYourGraveyard,
    /// "Add one mana of any of this creature's colors" — the source
    /// permanent's own computed colors (Katilda's granted Human ability);
    /// nothing off a colorless source.
    AnyColorOfSource,
    /// "Choose a color of a permanent you control. Add one mana of that color"
    /// (Meteor Crater). Colorless-only boards produce nothing.
    AnyColorAmongYourPermanents,
    /// Vivid — one mana of *each* color among your permanents (Bloom Tender).
    ///
    /// ⚠ Not [`Self::AnyColorAmongYourPermanents`], which adds one mana chosen
    /// from that set. They agree only at one color, which is where a mistake
    /// would hide. No choice, so no seat is asked; colorless board, no mana.
    OneOfEachColorAmongYourPermanents,
    /// "For each color among monocolored permanents you control, add one mana
    /// of that color" (Tarnation Vista): Vivid's set over monocolored ones.
    OneOfEachColorAmongYourMonocoloredPermanents,
    /// "For each color among the exiled cards used to craft this creature,
    /// add one mana of that color" (Sunbird Effigy): Vivid's shape over the
    /// cards exiled with the source.
    OneOfEachColorAmongExiledWithSource,
    /// Player chooses a color, then adds mana of that color equal to their
    /// devotion to it (CR 700.5). Nykthos, Shrine to Nyx's second ability.
    DevotionOfChosenColor,
    /// The inner payload with every pip tagged with `restriction` ("Spend this
    /// mana only to …"), colorless too (`add_restricted_colorless`). Strixhaven
    /// school mana, Powerstone tokens.
    Restricted(Box<ManaPayload>, SpendRestriction),
    /// Like `Restricted`, but "only to cast a creature spell of the chosen type,
    /// and that spell can't be countered" (Cavern of Souls); the type is read off
    /// the source at resolution, unrestricted with none.
    RestrictedToChosenType(Box<ManaPayload>),
    /// Like `RestrictedToChosenType`, without Cavern's uncounterable rider
    /// (Unclaimed Territory — "creature spell of the chosen type").
    RestrictedToChosenTypePlain(Box<ManaPayload>),
    /// `RestrictedToChosenTypePlain` that also funds abilities of creatures
    /// of the chosen type (Secluded Courtyard's "or activate an ability of a
    /// creature or creature card of the chosen type").
    RestrictedToChosenTypeOrAbility(Box<ManaPayload>),
    /// Kindred-inclusive sibling of `RestrictedToChosenTypeOrAbility`: any spell
    /// or source carrying the type (`SpendRestriction::TypeSpellOrItsAbility`, CR
    /// 308.3). Eclipsed Realms.
    RestrictedToChosenTypeSpellOrAbility(Box<ManaPayload>),
    /// `RestrictedToChosenTypePlain` that also funds any legendary creature
    /// spell (The Ringhart Crest's "a creature spell of the chosen type or a
    /// legendary creature spell").
    RestrictedToChosenTypeOrLegendary(Box<ManaPayload>),
    /// Like `RestrictedToChosenType`, restricted to monocolored spells of the
    /// source's chosen color (Throne of Eldraine). Unrestricted with none.
    RestrictedToChosenColorMono(Box<ManaPayload>),
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

/// Kinds of game events a trigger can watch for; mirrors [`GameEvent`].
///
/// `#[repr(u8)]` so [`EventKind::bit`] can read the tag as the first byte
/// (RFC 2195): the dispatcher masks a batch's kinds into a `u128` per
/// (permanent, trigger) pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EventKind {
    /// A permanent entered the battlefield.
    EntersBattlefield,
    /// A creature died (hit a graveyard from the battlefield).
    CreatureDied,
    /// CR 700.4 — a creature or artifact died (an artifact creature counts once).
    /// Judge Magister Gabranth; with `once_per_turn`, G'raha Tia. Use a
    /// `Predicate` filter for "another".
    CreatureOrArtifactDied,
    /// Any permanent died (`GameEvent::PermanentDied`); filter the subject with
    /// `Predicate::EntityMatches { what: TriggerSource, .. }` (Carth the Lion).
    PermanentDied,
    /// A creature was sacrificed (CR 701.16; distinct from dying — Mortician
    /// Beetle). `Effect::Sacrifice` emits it before `CreatureDied`.
    CreatureSacrificed,
    /// Any permanent was sacrificed (CR 701.16) — "whenever you sacrifice a
    /// permanent" (Korvold, Mayhem Devil). A creature fires both this and
    /// `CreatureSacrificed`.
    PermanentSacrificed,
    /// Any permanent left the battlefield.
    PermanentLeavesBattlefield,
    /// CR 701.8 — a spell or ability (not combat damage or an SBA) destroyed a
    /// permanent. The actor is the destroyer's controller, so `YourControl` +
    /// `actor_is_opponent` is Karmic Justice.
    PermanentDestroyedByEffect,
    /// CR 603.6 — a creature left the battlefield without dying (to hand / exile
    /// / library). Dour Port-Mage. `CreatureDied` covers the graveyard exit.
    CreatureLeavesBattlefieldNotDying,
    /// A card was drawn.
    CardDrawn,
    /// The turn's first card drawn (CR 121). Fired once per turn per player;
    /// the trigger subject is the drawn card. "The first time each turn you
    /// draw a card" (God-Eternal Kefnet).
    FirstCardDrawnThisTurn,
    /// A card was discarded.
    CardDiscarded,
    /// A card was put into a graveyard from a hand — discarded, or moved
    /// there by an effect (`GameEvent::CardPutIntoGraveyardFromHand`). Desert
    /// Warfare's "a Desert card is put into your graveyard from your hand".
    PutIntoGraveyardFromHand,
    /// An opponent's spell or ability made a player discard (CR 701.9 +
    /// `GameState.resolution_causer`), keyed on the discarder — `YourControl` =
    /// Spiritual Focus. Emitted with `CardDiscarded`.
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
    /// CR 701.43d — the creature was exerted as it attacked. Exert is an optional
    /// attack cost (CR 508.1g) and its "when you do" is a linked ability (CR
    /// 607.2h), so the bonus rides this, not `Attacks`. Glorybringer.
    Exerted,
    /// CR 508 — "whenever you attack": once per combat when the player declares
    /// any attackers (Razorkin Hordecaller). Dispatched from `declare_attackers`.
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
    /// CR 701.19 — "whenever this creature regenerates" (Skeleton
    /// Scavengers' counter rider). Fires whenever a regeneration shield is
    /// applied, whatever granted it.
    Regenerated,
    /// CR 509.3g — "attacks and isn't blocked": once per unblocked attacker after
    /// `declare_blockers`; the counterpart of `BecomesBlocked`.
    AttacksAndIsntBlocked,
    /// Combat damage was dealt to a player by a creature.
    DealsCombatDamageToPlayer,
    /// "Whenever this deals damage to a player", combat or not (Soltari
    /// Visionary); the player is bound to slot 0.
    DealsDamageToPlayer,
    /// A creature dealt combat damage to a planeswalker, keyed on the dealer; the
    /// planeswalker is the default target slot (Vraska's Assassin).
    DealsCombatDamageToPlaneswalker,
    /// The listener's controller was dealt combat damage, keyed on the recipient
    /// (Risona). Amount via `TriggerEventAmount`.
    ControllerDealtCombatDamage,
    /// Combat damage was dealt to a creature by a creature.
    DealsCombatDamageToCreature,
    /// "Whenever this deals damage to a creature", combat or not (Neko-Te); the
    /// creature is bound to slot 0.
    DealsDamageToCreature,
    /// "Whenever this deals combat damage" to anything — the union of the player
    /// and creature kinds (Descendant of Kiyomaro); recipient in slot 0.
    DealsCombatDamage,
    /// "Whenever this permanent deals damage" — recipient- *and*
    /// combat-agnostic (Kiyomaro, First to Stand). Fires from the combat
    /// steps and the non-combat damage funnel alike.
    DealsDamage,
    /// CR 702.130 Enrage — a permanent was dealt damage of any kind, keyed on the
    /// recipient (unlike `DealsCombatDamageToCreature`). Amount via
    /// `Value::TriggerEventAmount`; `SelfSource` for enrage, wider scopes for
    /// "a creature you control is dealt damage".
    DealtDamage,
    /// A permanent was dealt combat damage — combat-only `DealtDamage`, keyed on
    /// the recipient (Souls of the Faultless). Amount via `TriggerEventAmount`.
    DealtCombatDamage,
    /// CR 510 / 119 — a player was dealt noncombat damage, keyed on that player
    /// (`OpponentControl` = Chandra's Spitfire). Amount via `TriggerEventAmount`.
    PlayerDealtNoncombatDamage,
    /// Your source dealt noncombat damage to a creature equal to its toughness
    /// (Taii Wakeen), keyed on the creature; amount via `TriggerEventAmount`.
    YourSourceDealtNoncombatDamageEqualToToughness,
    /// A player was dealt damage (combat or not). Pair with
    /// `EventScope::YourSourceDamagedOpponent` for "whenever a source you
    /// control deals damage to an opponent" (Quest for Pure Flame).
    PlayerDamaged,
    /// Your instant or sorcery dealt damage, once per resolution however many it
    /// hit (Blaze Commando). Amount via `TriggerEventAmount`.
    YourInstantOrSorceryDealtDamage,
    /// A controller's instant or sorcery dealt damage to a player, once per
    /// player (unlike `YourInstantOrSorceryDealtDamage`), binding them as the
    /// subject so `ControlledByTriggerPlayer` reads "that player controls" (Satyr
    /// Firedancer).
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
    /// CR 715.2e — a Saga's final chapter resolved (Narci, Fable Singer). Subject
    /// the Saga, amount its mana value, read before CR 715.4 sacrifices it.
    SagaFinalChapterResolved,
    /// CR 701.22 — "whenever you scry" only (Flamespeaker Adept); the same
    /// event as `ScriedOrSurveiled`, narrowed by its `surveil` flag.
    Scried,
    /// CR 701.25 — "whenever you surveil" only (Mirko, Dimir Spybug).
    Surveilled,
    /// CR 701.17 — the literal "mills" wording only (Zellix). `CardMilled` is the
    /// wider library-to-graveyard family that surveil also fires (Narcomoeba).
    Milled,
    /// CR 701.49 — a player completed a dungeon (the final room's ability
    /// resolved). "Whenever you complete a dungeon" (Dungeon Crawler).
    DungeonCompleted,
    /// CR 701.34 — a player proliferated, once per instance (a doubled
    /// proliferate fires twice). Ezuri, Stalker of Spheres.
    Proliferated,
    /// A player foraged (CR 701.61 — exiled three graveyard cards or sacrificed
    /// a Food; `GameEvent::Foraged`). The foraging player is the event actor.
    /// "Whenever you forage" (Corpseberry Cultivator).
    Foraged,
    /// CR 701.59 — a player collected evidence (as a cost or via
    /// `Effect::CollectEvidence`). Surveillance Monitor.
    EvidenceCollected,
    /// A player gave a promised gift (CR 702.165; `GameEvent::GiftGiven`). The
    /// gifting player is the event actor. "Whenever you give a gift" (Jolly
    /// Gerbils).
    GiftGiven,
    /// A player got one or more poison counters (`GameEvent::PoisonAdded`) —
    /// the player half of All Will Be One's "counters on a permanent or
    /// player". Event amount = counters added.
    PoisonAdded,
    /// CR 701.54 — the Ring tempted a player; the chosen bearer is the subject
    /// (Call of the Ring).
    RingTempted,
    /// CR 700.14 — "whenever you expend N": the filter carries
    /// `Predicate::ExpendReached(n)`, so it fires as the total first reaches N.
    Expend,
    /// CR 717.2 — a Class gained a level. Scope `EventScope::This` and filter on
    /// `Predicate::SourceClassLevelIs(N)` for "becomes level N".
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
    /// Counters of a kind were removed (one event, the total as amount). Chandra,
    /// Fire Artisan.
    CounterRemoved(CounterType),
    /// A counter of *any* kind was added to a permanent (CR 122). Fires once
    /// per `CounterAdded` event regardless of counter type — "whenever one or
    /// more counters are put on a creature you control" (Stalwart Successor).
    AnyCounterAdded,
    /// An ability was activated.
    AbilityActivated,
    /// An ability was activated, with the [`ActivationTrait`] named. Also an
    /// `AbilityActivated`. One payload variant, so the four share a kind bit
    /// (`EventKind` is at its 128-tag `u128` mask).
    AbilityActivatedWith(ActivationTrait),
    /// A card left a player's graveyard, once per card (Strixhaven "one or more
    /// cards leave" payoffs fire per card — Living History).
    CardLeftGraveyard,
    /// A card was put into exile from any zone, per card. With `once_per_turn` +
    /// `IsTurnOf(You)`: Stonebinder's Familiar.
    CardExiled,
    /// "One or more cards are put into exile from [zones]" — the origin-scoped
    /// sibling of `CardExiled` (CR 400.7), payload a mask of [`exile_from`]
    /// origins (Ketramose). One kind-mask bit for every mask.
    CardExiledFrom(u8),
    /// A permanent became the target of a spell or ability, once per target at
    /// announce time. The trigger source must be the targeted permanent
    /// (`event_matches_spec`); the scope refines on the caster. SOS Tenured
    /// Concocter.
    BecameTarget,
    /// CR 601.2c — "whenever a player chooses one or more targets": one event
    /// per targeting decision, not per targeted object (Psychic Battle).
    ChoseTargets,
    /// CR 702.29c — a card was cycled, emitted by `GameState::cycle_card` beside
    /// `CardDiscarded` so cycle triggers skip plain discards. The ability usually
    /// lives on the cycled card and triggers from its new zone; `SelfSource` =
    /// this card, `YourControl` = any of yours.
    CardCycled,
    /// A card was milled (`GameEvent::CardMilled`); with `SelfSource` it fires
    /// from the graveyard (Narcomoeba).
    CardMilled,
    /// DSK manifest dread; the subject is the card put into the graveyard
    /// (Paranormal Analyst), the manifester the actor.
    ManifestedDread,
    /// CR 702.108 — a permanent became untapped (Inspired). Fired once per
    /// permanent that flips tapped→untapped during the untap step. The
    /// triggering permanent is the event subject.
    BecomesUntapped,
    /// A permanent became tapped (Magda, Brazen Outlaw). The tapped
    /// permanent is the event subject; matched to `GameEvent::PermanentTapped`.
    Tapped,
    /// CR 702.122 / 702.171 — "whenever this creature crews a Vehicle or saddles
    /// a Mount", fired from the crewer's side; the Vehicle / Mount is the subject
    /// (`Selector::TriggerSource`). `main_phase_only` is the printed "during your
    /// main phase" (Canyon Vaulter).
    CrewsOrSaddles { main_phase_only: bool },
    /// CR 702.26 — a permanent phased in. The phasing-in permanent is the
    /// event subject; matched to `GameEvent::PermanentPhasedIn`.
    PhasesIn,
    /// CR 702.26 — a permanent phased out; matched to
    /// `GameEvent::PermanentPhasedOut` (Ertai's Familiar).
    PhasesOut,
    /// CR 702.24 — a player didn't pay a permanent's cumulative upkeep, so it
    /// was sacrificed (Heart of Bogardan). `SelfSource`; the age counters are
    /// still readable off the death LKI.
    CumulativeUpkeepUnpaid,
    /// CR 800.4 — "when you gain control of this permanent from another
    /// player" (Risky Move). `SelfSource`; the trigger's controller must be
    /// the *new* controller, so it fires once for whoever just took it.
    GainedControlOfThis,
    /// CR 800.4 — "when you lose control of this permanent" (Duplicity), controlled
    /// by the seat that lost it. `yours`: an opponent gained control of any of your
    /// other permanents (Zidane); one mask bit for both.
    LostControl { yours: bool },
    /// CR 701.44 — a permanent explored (Wildgrowth Walker, Tishana's
    /// Wayfinder payoffs). The exploring permanent is the event subject;
    /// matched to `GameEvent::Explored`.
    Explored,
    /// CR 701.50 — a permanent connived (Glorious Purpose, Iron Monger,
    /// Ultron's "whenever a creature you control connives"). The conniver is
    /// the event subject; matched to `GameEvent::Connived`.
    Connived,
    /// CR 120.10 / 120.4a — a creature or planeswalker was dealt excess damage
    /// (subject the permanent, actor its controller, amount the excess).
    /// `noncombat_only` matches `ExcessNoncombatDamage` alone (Fall of Cair
    /// Andros); otherwise combat too (Aegar). One payload variant: the kind mask
    /// is full.
    DealtExcessDamage { noncombat_only: bool },
    /// CR 701.57 — the player discovered (subject); the discover value is
    /// `TriggerEventAmount`. Curator of Sun's Creation.
    Discovered,
    /// CR 701.37 — a permanent became monstrous (Fleecemane Lion, Nessian
    /// Wilds Ravager "when this becomes monstrous" triggers). The permanent
    /// is the event subject; matched to `GameEvent::BecameMonstrous`.
    BecameMonstrous,
    /// CR 107.16 — the player got {E}, once per `AddEnergy`; amount via
    /// `TriggerEventAmount`.
    EnergyGained,
    /// CR 705.1 — won a coin flip; with `clash`, CR 701.30d — won a clash (Marvo).
    /// One event per win, the player as subject. A payload because the `u128`
    /// mask is full.
    WonContest { clash: bool },
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
    /// "Whenever you roll a die's highest natural result" (Netherese
    /// Puzzle-Ward): once per die. Matched to `GameEvent::RolledNaturalMax`.
    RolledNaturalMax,
    /// CR 701.9 — the player discarded one or more cards, once per resolution;
    /// count via `TriggerEventAmount`.
    DiscardedOneOrMore,
    /// CR 303.4 — an Aura became attached; the attached-to permanent is the
    /// subject (`YourControl` needs it to be your creature). Siona, Captain of the
    /// Pyleas.
    AuraAttached,
    /// `AuraAttached` with the Aura as subject and any host (Eriette, the
    /// Beguiler), so the body can reach both.
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
    /// CR 111.10 — a token was created (subject); narrow the kind with a
    /// TriggerSource filter.
    TokenCreated,
    /// CR 709.5h — a Room door was unlocked (at cast-entry or via the unlock
    /// special action). Fired with the Room permanent as the subject.
    DoorUnlocked,
    /// DSK Eerie — a Room was fully unlocked (subject); pair with `YourControl`.
    RoomFullyUnlocked,
    /// CR 903.9 — your commander was put into the command zone from anywhere
    /// (Myth Unbound). `YourControl` = its owner.
    CommanderPutIntoCommandZone,
    /// MKM — "whenever you solve a Case" (Case File Auditor). Fired with the
    /// solved Case as the subject; pair with `EventScope::YourControl` so the
    /// solver's permanents trigger. Matched to `GameEvent::CaseSolved`.
    CaseSolved,
    /// A land card went to a graveyard from anywhere, once per simultaneous batch
    /// (The Gitrog Monster). `YourControl` = the graveyard's owner.
    LandPutIntoGraveyard,
    /// A card was put into a graveyard from anywhere; with `SelfSource` it fires
    /// from the graveyard (Emrakul's shuffle).
    PutIntoGraveyard,
    /// "When this card is put into your hand from your graveyard" (Golgari
    /// Brownscale); `SelfSource` fires from the hand.
    PutIntoHandFromGraveyard,
    /// CR 502.2 / 732 — day/night flipped; global, so pair with
    /// `EventScope::AnyPlayer`. Brimstone Vandal.
    DayNightChanged,
    /// CR 700.13 — `who` committed a crime (targeted an opponent, their
    /// permanents/cards, or their spells), once per spell or ability. Pair with
    /// `YourControl` (Kaervek).
    CommittedCrime,
    /// CR 702.170 — "when this card becomes plotted", a `SelfSource` trigger from
    /// exile dispatched by `plot_card` (Aloe Alchemist).
    BecomesPlotted,
    /// CR 701.23 — a player searched their library, found or not (Ob Nixilis,
    /// Unshackled with `OpponentControl`).
    PlayerSearchedLibrary,
    /// CR 103.2c — a spell or ability caused a player to shuffle their library.
    /// The shuffling player is the event subject (Psychogenic Probe).
    LibraryShuffled,
    /// A permanent was returned to its owner's hand (Azorius Aethermage); subject
    /// the card, `YourControl` = whose hand.
    PermanentReturnedToHand,
    /// CR 605 — a permanent was tapped to pay a mana ability's `{T}` cost.
    /// The tapped permanent is the event subject (Extraplanar Lens).
    TappedForMana,
    /// CR 701.6 — a spell was countered. Actor the counterer's controller
    /// (`YourControl` = Lullmage Mentor), subject the countered card.
    SpellCountered,
    /// CR 726.3 — a player became the monarch. The event's player is the
    /// new monarch, so `EventScope::OpponentControl` reads "an opponent
    /// becomes the monarch" (Knights of the Black Rose).
    BecameMonarch,
    /// CR 121 — a player's `n`th draw of the turn (subject the drawn card), per
    /// player. Gleaming Splendor `(2)`, Astrologian's Planisphere `(3)`. Emitted
    /// for `n` = 2, 3 and 5 only (the first draw has its own kind); one mask bit
    /// for every `n`.
    NthCardDrawnThisTurn(u8),
    /// CR 800.4a — "When enchanted player loses the game" (Curse of
    /// Vengeance): queued for each Aura attached to a player as that player
    /// leaves, before the Aura goes; the event amount is the counters on it.
    EnchantedPlayerLeftGame,
    /// CR 800.4a — "Whenever a player loses the game" (Blood Tyrant): queued
    /// for every permanent carrying it as any player leaves; the departing
    /// player's own permanents are gone with them.
    PlayerLeftGame,
    /// CR 615 — damage to a player was prevented (Selfless Squire); amount = the
    /// prevented damage.
    DamageToPlayerPrevented,
    /// "Cards are put into exile from your hand or a spell or ability you control
    /// exiles permanents" (Ranar the Ever-Watchful): one event per player per
    /// batch, synthesized from the exile tally; amount = the card count.
    CardsExiledFromHandOrByYou,
    /// CR 701.12 — a creature fought, one event per fighter (Foe-Razer Regent).
    Fights,
    /// A keyword action was performed — the action is the payload. ⚠ The
    /// last bit of the `u128` kind mask (tag 127): a new action-shaped kind
    /// is a new [`KeywordAct`], not a new variant here.
    Performed(KeywordAct),
}

/// The action an [`EventKind::Performed`] watches for. One mask bit covers
/// them all, so `event_kind_matches` reads the payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeywordAct {
    /// CR 701.16a — a player investigated. `first_only`: only their first this
    /// turn, counted even if the listener wasn't there (Erdwal Illuminator).
    Investigated { first_only: bool },
    /// CR 702.110b — a creature exploited a creature (its exploit trigger
    /// sacrificed one); the exploiter is the subject (Colonel Autumn), the
    /// exploited creature's last-known power the event amount.
    Exploited,
    /// [`Exploited`](Self::Exploited) where the exploited creature wasn't a
    /// Human (Henry Wu, InGen Geneticist).
    ExploitedNonHuman,
    /// [`Exploited`](Self::Exploited) where the exploited creature wasn't a
    /// token (Skull Skaab).
    ExploitedNontoken,
    /// CR 702.154c — an attacking creature enlisted a creature; the attacker
    /// is the subject (Aradesh, the Founder).
    Enlisted,
    /// CR 702.112b — a permanent became renowned; it is the subject (Valeron
    /// Wardens' "whenever a creature you control becomes renowned").
    BecameRenowned,
    /// CR 706.2 — one die's result (amount), per die; the whole roll is
    /// `RolledDice`. With `DieResultAtLeast`: Mr. House.
    DieResult,
    /// A player bent (`None` = any): airbend (CR 701.65b), earthbend (701.66b,
    /// its return trigger created), waterbend (701.67c, cost paid), firebend
    /// (702.189b, ability resolved). The bender is the actor (Avatar Aang).
    Bent(Option<BendKind>),
    /// An instant or sorcery's resolution made a player gain life (Firesong and
    /// Sunspeaker). Subject the spell (filter via `EntityMatches` on
    /// `TriggerSource`), actor the gainer, amount the total; one per gainer.
    SpellCausedLifeGain,
    /// CR 702.30a — a permanent's echo cost was paid at its controller's
    /// upkeep; the permanent is the subject ("When this creature's echo cost
    /// is paid" — Shah of Naar Isle).
    EchoPaid,
    /// A creature became tapped to pay a teamwork cost; it is the subject
    /// (Agent Maria Hill).
    TappedForTeamwork,
    /// CR 701.22a — a player chose to put one or more cards on the bottom
    /// while scrying; the scryer is the actor, the count bottomed the event
    /// amount (The Temporal Anchor).
    ScryBottomed,
    /// CR 123.3 — a player put a sticker on an object; the player is the actor,
    /// the object the subject (Wee Champion).
    StickerPlaced,
    /// Cards were put into a library from another zone (not a scry or reorder):
    /// one event per batch, amount the count, actor the first library's owner
    /// (Wan Shi Tong).
    CardsPutIntoLibrary,
    /// CR 701.51c — a player opened an Attraction (it entered from their
    /// Attraction deck); the opener is the actor, the Attraction the subject.
    OpenedAttraction,
    /// CR 702.159b — a player claimed an Attraction's prize; the player is the
    /// actor, the Attraction the subject.
    ClaimedPrize,
}

/// The four bending actions [`KeywordAct::Bent`] tells apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BendKind {
    Air,
    Earth,
    Fire,
    Water,
}

impl BendKind {
    /// This kind's bit in `Player::bends_this_turn`.
    pub const fn bit(self) -> u8 {
        match self {
            BendKind::Air => 1,
            BendKind::Earth => 2,
            BendKind::Fire => 4,
            BendKind::Water => 8,
        }
    }
}

impl EventKind {
    /// This kind's bit in a kind mask: `1 << tag`, `const` so tables fold.
    /// Payload variants share one bit: a mask answers "could this kind match",
    /// never the payload.
    #[inline]
    pub const fn bit(&self) -> u128 {
        // SAFETY: `#[repr(u8)]` lays every variant out as a `u8` tag followed
        // by its fields (RFC 2195), so the first byte is the discriminant.
        let tag = unsafe { *(self as *const EventKind as *const u8) };
        debug_assert!(tag < 128, "EventKind has outgrown a u128 kind mask");
        1u128 << tag
    }

    /// [`bit`](Self::bit) folded to one word (kinds `i` and `i + 64` share a
    /// bit). Sound as a skip — enough for the per-definition trigger-kind memo
    /// in the spare word of [`crate::card::CardMemo`].
    #[inline]
    pub const fn fold(&self) -> u64 {
        Self::fold_bits(self.bit())
    }

    /// The fold of a [`bit`](Self::bit) mask, for a mask already in hand.
    #[inline]
    pub const fn fold_bits(bits: u128) -> u64 {
        (bits as u64) | ((bits >> 64) as u64)
    }
}

/// What [`EventKind::AbilityActivatedWith`] asks of the activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationTrait {
    /// One or more permanents were sacrificed to pay for it (Ashnod the
    /// Uncaring).
    Sacrifice,
    /// Life was paid to activate it (Verrak, Warped Sengir); the event amount
    /// is the life paid.
    LifePaid,
    /// CR 702.177 — an exhaust ability ("whenever you activate an exhaust
    /// ability" — Adrenaline Jockey).
    Exhaust,
    /// CR 702.108 — an adapt ability (Gyre Engineer), detected structurally
    /// from the activated ability's counter-check shape (`Effect::is_adapt`).
    Adapt,
    /// CR 702.49 — a ninjutsu ability (Satoru Umezawa).
    Ninjutsu,
    /// CR 606.1 — a loyalty ability (Way of the Paradox).
    Loyalty,
    /// CR 702.193 — a power-up ability (Marvel Boy, Noh-Varr).
    PowerUp,
    /// CR 702.128 / 702.129 — an embalm or eternalize ability (Vizier of the
    /// Anointed), detected structurally (`ActivatedAbility::is_embalm_like`).
    Embalm,
    /// CR 605.1a — a mana ability without `{T}` in its cost (Elrond's
    /// "an ability of a creature" counts mana abilities; the `{T}` ones are
    /// `EventKind::TappedForMana`).
    ManaWithoutTap,
    /// Mana from a Treasure was spent to activate it (Vazi, Keen Negotiator).
    TreasureMana,
    /// CR 702.142 — a boast ability (Frenzied Raider), detected structurally
    /// (`ActivatedAbility::is_boast`).
    Boast,
}

/// The zones [`EventKind::CardExiledFrom`] distinguishes (a bitmask).
pub mod exile_from {
    pub const BATTLEFIELD: u8 = 1;
    pub const GRAVEYARD: u8 = 2;
    pub const LIBRARY: u8 = 4;
    /// Set beside `BATTLEFIELD` on a permanent exiled to pay a craft cost
    /// ("exiled from the battlefield while you're activating a craft ability").
    pub const CRAFT: u8 = 8;
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
    /// The trigger fires from its owner's graveyard, not the battlefield
    /// (Bloodghast); the dispatcher walks graveyards for this scope and the owner
    /// is the controller.
    FromYourGraveyard,
    /// [`FromYourGraveyard`](Self::FromYourGraveyard) on any player's event, so a
    /// step trigger fires on every player's step (Kami of Transience).
    FromYourGraveyardAnyPlayer,
    /// A `PlayerDamaged` event whose source is controlled by the trigger's
    /// controller and whose damaged player is an opponent of them ("a source
    /// you control deals damage to an opponent" — Quest for Pure Flame).
    YourSourceDamagedOpponent,
    /// Combat damage to an opponent from a creature you control (Lolth's emblem).
    /// One trigger per creature; an intervening `if` the first resolution
    /// falsifies collapses them.
    YourCreatureCombatDamagedOpponent,
    /// [`YourSourceDamagedOpponent`](Self::YourSourceDamagedOpponent) with the
    /// printed "other than this <permanent>" exclusion (Talon of Pain): the
    /// damage event's own source must not be the trigger's source.
    YourOtherSourceDamagedOpponent,
    /// Mirror of `YourSourceDamagedOpponent`: an opponent's source damaged you
    /// (Michiko Konda). The dealer's controller is bound as the trigger player.
    OpponentSourceDamagedYou,
    /// With `EventKind::DealtDamage`: an opponent's source damaged a permanent you
    /// control (Elesh Norn). Subject the permanent; the dealer's controller is
    /// `PlayerRef::TriggerEventPlayer`.
    OpponentSourceDamagedYourPermanent,
    /// With `EventKind::BecameTarget`: any permanent you control (not only the
    /// source) became the target of an opponent's spell or ability. Battle
    /// Mammoth.
    YourPermanentTargetedByOpponent,
    /// **You** or a permanent you control becomes the target of a spell or
    /// ability an opponent controls — once per such target (Leovold, Rayne;
    /// their rulings). Hears `PlayerBecameTarget` as well as `BecameTarget`.
    YouOrYourPermanentTargetedByOpponent,
    /// A creature **you control** or a creature **spell** you control becomes
    /// the target of a spell or ability an **opponent** controls (Surrak,
    /// Elusive Hunter). Used with `EventKind::BecameTarget`.
    YourCreatureOrCreatureSpellTargetedByOpponent,
    /// A creature the source's controller controls (including the source)
    /// becomes the target of any spell or ability, friendly or hostile.
    /// Used with `EventKind::BecameTarget` — Nadu, Winged Wisdom.
    YourCreatureTargeted,
    /// With `EventKind::BecameTarget`: your spell or ability targeted an
    /// opponent's creature (Willbreaker); the creature is `TriggerSource`.
    OpponentCreatureTargetedByYou,
    /// CR 115.1 — a player or a battlefield permanent becomes the target of
    /// an *ability* the source's controller controls (a spell's targets
    /// don't count; nor do cards in other zones). Loki, God of Mischief.
    PlayerOrPermanentTargetedByYourAbility,
    /// With `EventKind::Attacks`: an opponent's creature attacks you or your
    /// planeswalker (bare "attacks you" is
    /// `ControllerAttackedDirectlyByOpponent`). The attacker's controller is
    /// bound to `PlayerRef::Target(0)` (Coveted Jewel).
    ControllerAttackedByOpponent,
    /// An opponent's creature attacks a planeswalker you control (not you) —
    /// Mila, Crafty Companion. Same dispatcher as `ControllerAttackedByOpponent`.
    ControllerPlaneswalkerAttackedByOpponent,
    /// CR 508.1 — "whenever a player attacks one of your opponents" (Breena),
    /// once per declaration per attacked opponent. Attacker in `Target(0)`,
    /// attacked opponent as `Triggerer`.
    OpponentOfYoursAttacked,
    /// CR 508.1 — "whenever a player attacks" (Mirkwood Trapper), once per
    /// declaration. The attacker is `Target(0)` and the trigger source; the
    /// declaration is complete at dispatch, so the filter can ask whom.
    AnyPlayerAttacks,
    /// CR 508.1 — "whenever you attack a player" (Karazikar), once per attacked
    /// player, who is `Triggerer` and the trigger player
    /// (`ControlledByTriggerPlayer`).
    YouAttackedPlayer,
    /// The creature the source Aura enchanted died (per
    /// `GameState.auras_at_death`) — Minion's Return.
    EnchantedBySource,
    /// A permanent was tapped by your effect (Sharae): keyed on the tapper
    /// (`PermanentTapped.actor`), unlike `Tapped` / `YourControl`. Subject the
    /// tapped permanent.
    YouTapped,
    /// "Whenever you put one or more counters on …" — a `CounterAdded` whose
    /// placer (CR 122.6) is the trigger's controller, whoever controls the
    /// permanent; a `.with_filter` on `TriggerSource` restricts the subject.
    YouPutCounters,
    /// "Whenever an opponent puts one or more counters on a creature they
    /// control" (Bold Plagiarist): a `CounterAdded` whose placer (CR 122.6) is
    /// an opponent of the trigger's controller and controls the permanent.
    OpponentPutsCountersOnTheirOwn,
    /// CR 506.3 — an opponent's creature attacks you directly (Orim's Prayer);
    /// planeswalker attacks don't count. Same dispatcher as
    /// `ControllerAttackedByOpponent`.
    ControllerAttackedDirectlyByOpponent,
    /// [`YourSourceDamagedOpponent`](Self::YourSourceDamagedOpponent) with
    /// ANY damaged player, its controller included ("whenever a source you
    /// control deals 5 or more damage to a player" — Dragonborn Champion).
    YourSourceDamagedAnyPlayer,
    /// [`YourSourceDamagedOpponent`](Self::YourSourceDamagedOpponent) read from
    /// the owner's graveyard (Asgardian Inspiration); "you" is the owner.
    FromYourGraveyardSourceDamagedOpponent,
    /// [`OpponentSourceDamagedYou`](Self::OpponentSourceDamagedYou) from ANY
    /// source, yours included ("whenever a source of the chosen color deals
    /// damage to you" — Circle of Affliction). The dealer is the subject.
    SourceDamagedYou,
    /// A source the trigger's controller controls dealt damage to them ("a
    /// source you control deals … damage to you" — Sahir). A source in its
    /// owner's command zone is that player's (Eminence pings).
    YourSourceDamagedYou,
}

impl EventScope {
    /// The trigger functions only while its card is in its owner's graveyard
    /// (CR 603.3d zone scoping) — the one predicate every graveyard walk and
    /// the battlefield walk's skip read, so they cannot disagree.
    pub fn from_graveyard(&self) -> bool {
        matches!(
            self,
            Self::FromYourGraveyard
                | Self::FromYourGraveyardAnyPlayer
                | Self::FromYourGraveyardSourceDamagedOpponent
        )
    }
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
    /// CR 603.3d — "triggers only once each turn" (and once per batch), tracked
    /// in `GameState::triggered_once_per_turn_used`. `#[serde(default)]` for
    /// snapshot back-compat. Dramatic Finale.
    #[serde(default)]
    pub once_per_turn: bool,
    /// "Whenever one or more …" (CR 603.2c): one trigger per batch of
    /// simultaneous events, however many match, with no per-turn cap
    /// (Attuned Hunter's "one or more cards leave your graveyard").
    #[serde(default)]
    pub once_per_batch: bool,
    /// With `once_per_batch` on combat damage: one fire for all players dealt
    /// damage in the sub-step — "one or more players" (Contaminant Grafter).
    #[serde(default)]
    pub batch_across_players: bool,
    /// With `once_per_batch` on `Attacks`: one fire per defending player (Neyali,
    /// ruling), where plain "attack" fires once per declaration.
    #[serde(default)]
    pub batch_per_defender: bool,
    /// With `once_per_batch` on combat damage: the amount is the batch's total to
    /// that player, not the first dealer's (Anowon, the Ruin Thief).
    #[serde(default)]
    pub batch_sums_damage: bool,
    /// With `once_per_batch`: the amount is the number of card types among the
    /// batch's cards (Polluted Cistern).
    #[serde(default)]
    pub batch_counts_card_types: bool,
    /// With `once_per_batch`: the amount is how many batch events match, filter
    /// included (The Wise Mothman).
    #[serde(default)]
    pub batch_counts_subjects: bool,
    /// "This ability triggers only N times each turn" counted per event
    /// subject (Nadu's granted trigger is per creature). `None` = uncapped.
    #[serde(default)]
    pub per_subject_cap: Option<u8>,
    /// The event's actor must be an opponent of the trigger's controller, for
    /// scopes that don't already gate the actor (Opaline Sliver).
    #[serde(default)]
    pub actor_is_opponent: bool,
    /// "…becomes tapped, if it isn't being declared as an attacker" — excludes
    /// the CR 508.1f attacker tap from an `EventKind::Tapped` trigger (Verity
    /// Circle). Defaults to false. Only meaningful for `Tapped`.
    #[serde(default)]
    pub exclude_attacker_taps: bool,
    /// "Activates an ability without {T} in its cost" (Haunting Wind):
    /// `AbilityActivated` only; tap-cost activations go to the `Tapped` trigger.
    #[serde(default)]
    pub exclude_tap_cost_abilities: bool,
    /// Restricts a damage event by its dealer (Cabal Slaver's Goblin), which
    /// `filter` can't reach since `TriggerSource` is the damaged player. Damage
    /// kinds only.
    #[serde(default)]
    pub dealer_filter: Option<crate::card::SelectionRequirement>,
    /// Restricts a targeting event by the object that declared the target ("the
    /// target of an Aura spell" — Fugitive Druid), which `filter` can't reach.
    /// `EventKind::BecameTarget` only.
    #[serde(default)]
    pub causer_filter: Option<crate::card::SelectionRequirement>,
    /// CR 113.6b — where this trigger functions, when that isn't just the
    /// zone its card's abilities ordinarily work from. Eminence.
    #[serde(default)]
    pub zone: TriggerZone,
    /// Magecraft's "cast or copy": also fires when the controller copies a
    /// matching spell (a copy isn't cast, CR 707.10), via
    /// `fire_spell_cast_triggers`' copy-only mode.
    #[serde(default)]
    pub or_copy: bool,
    /// "Whenever a spell or ability causes **its controller** to shuffle"
    /// (Widespread Panic): only a `LibraryShuffled` whose cause is the
    /// shuffling player's own spell or ability matches.
    #[serde(default)]
    pub self_caused: bool,
    /// "…become the target of an **activated** ability" (Professor Hojo): the
    /// targeting object is an activated ability on the stack, from any zone
    /// (a spell or a triggered ability doesn't count).
    #[serde(default)]
    pub by_activated_ability: bool,
    /// With `once_per_batch` on `LifeLost`: counts a player only if their batch
    /// total is exactly this (Ob Nixilis, Captive Kingpin; the ruling sums combat
    /// damage the engine emits per attacker).
    #[serde(default)]
    pub batch_life_lost_exactly: Option<u32>,
    /// "…deals combat damage to a player **or battle**": a
    /// `DealsCombatDamageToPlayer` trigger that also fires off combat damage
    /// to a battle (CR 310.6), the battle bound to slot 0.
    #[serde(default)]
    pub or_battle: bool,
}

/// CR 113.6b — zones an ability functions from; the command-zone axis.
/// Eminence (CR 207.2c) is "command zone or battlefield"; Oloro's second
/// upkeep trigger is "command zone" alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TriggerZone {
    /// Wherever the ability would ordinarily function — the battlefield for a
    /// permanent card, a graveyard for a `FromYourGraveyard` scope.
    #[default]
    Printed,
    /// …and also from its owner's command zone (Eminence).
    CommandZoneToo,
    /// From its owner's command zone and nowhere else.
    CommandZoneOnly,
    /// CR 702.62b — from exile while the card is suspended (it has suspend
    /// and a time counter) and nowhere else: Nihilith's "if this card is
    /// suspended". Gathered by the dispatch's suspended-exile walk.
    WhileSuspended,
    /// From exile while the card was exiled by its own ability
    /// (`exiled_with` is its own id) and nowhere else: the ability Cosima,
    /// God of the Voyage "gains" as it exiles itself.
    WhileSelfExiled,
    /// From exile, however the card got there, and nowhere else — "if this
    /// card is exiled" (Senu, Keen-Eyed Protector). Gathered by the
    /// dispatch's exile walk behind `CardPile::has_exile_trigger`.
    WhileExiled,
}

impl TriggerZone {
    /// True if the trigger functions while its card is in the command zone.
    pub fn in_command_zone(self) -> bool {
        matches!(self, Self::CommandZoneToo | Self::CommandZoneOnly)
    }
    /// True if the trigger does *not* function from the battlefield: the
    /// command zone is the only place (Oloro), or exile while suspended.
    pub fn command_zone_only(self) -> bool {
        matches!(self, Self::CommandZoneOnly | Self::WhileSuspended | Self::WhileSelfExiled | Self::WhileExiled)
    }
}

impl EventSpec {
    pub fn new(kind: EventKind, scope: EventScope) -> Self {
        Self {
            kind,
            scope,
            zone: TriggerZone::Printed,
            filter: None,
            once_per_turn: false,
            once_per_batch: false,
            batch_across_players: false,
            batch_per_defender: false,
            batch_sums_damage: false,
            batch_counts_card_types: false,
            batch_counts_subjects: false,
            per_subject_cap: None,
            actor_is_opponent: false,
            exclude_attacker_taps: false,
            exclude_tap_cost_abilities: false,
            dealer_filter: None,
            causer_filter: None,
            or_copy: false,
            self_caused: false,
            by_activated_ability: false,
            batch_life_lost_exactly: None,
            or_battle: false,
        }
    }
    /// "…to a player or battle" — see `or_battle`.
    pub fn or_battle(mut self) -> Self {
        self.or_battle = true;
        self
    }
    /// "…the target of an activated ability" — see `by_activated_ability`.
    pub fn by_activated_ability(mut self) -> Self {
        self.by_activated_ability = true;
        self
    }
    /// "…causes its controller to shuffle" (Widespread Panic) — see `self_caused`.
    pub fn self_caused(mut self) -> Self {
        self.self_caused = true;
        self
    }
    /// "Whenever you cast or copy …" (Magecraft) — see `or_copy`.
    pub fn or_copy(mut self) -> Self {
        self.or_copy = true;
        self
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
    /// Eminence (CR 113.6b) — also functions from its owner's command zone.
    pub fn in_command_zone(mut self) -> Self {
        self.zone = TriggerZone::CommandZoneToo;
        self
    }
    /// CR 113.6b — functions from the command zone and nowhere else
    /// (Oloro, Ageless Ascetic's second upkeep trigger).
    pub fn command_zone_only(mut self) -> Self {
        self.zone = TriggerZone::CommandZoneOnly;
        self
    }
    /// CR 702.62b — functions from exile while suspended, nowhere else
    /// (Nihilith).
    pub fn while_suspended(mut self) -> Self {
        self.zone = TriggerZone::WhileSuspended;
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
    /// Once per batch, with the number of card types among the batch's
    /// cards as the event amount (Polluted Cistern).
    pub fn once_per_batch_counting_card_types(mut self) -> Self {
        self.once_per_batch = true;
        self.batch_counts_card_types = true;
        self
    }
    /// "Whenever one or more …" — once per batch of simultaneous events.
    pub fn once_per_batch(mut self) -> Self {
        self.once_per_batch = true;
        self
    }
    /// "…deal combat damage to one or more players" — once per combat-damage
    /// sub-step however many players were hit (CR 603.2c).
    pub fn once_per_batch_across_players(mut self) -> Self {
        self.once_per_batch = true;
        self.batch_across_players = true;
        self
    }
    /// "Whenever one or more … attack a player" — once per defending player.
    pub fn once_per_batch_per_defender(mut self) -> Self {
        self.once_per_batch = true;
        self.batch_per_defender = true;
        self
    }
    /// "One or more … each lose exactly `n` life" — see `batch_life_lost_exactly`.
    pub fn once_per_batch_losing_exactly(mut self, n: u32) -> Self {
        self.once_per_batch = true;
        self.batch_life_lost_exactly = Some(n);
        self
    }
    /// Once per batch, with the batch's summed damage as the event amount.
    pub fn once_per_batch_summing_damage(mut self) -> Self {
        self.once_per_batch = true;
        self.batch_sums_damage = true;
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
    /// Sacrifice the tokens at end of combat (Kavaron Harrier, Gemini Engine).
    SacrificeAtEndOfCombat,
    /// Sacrifice the tokens at the beginning of the next end step — Mobilize
    /// (CR 702.181a), War Effort. After combat, so they leave not attacking.
    SacrificeAtNextEndStep,
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

/// `serde(default)` for a count field that reads 1 when absent.
fn one_u32() -> u32 {
    1
}

/// The root instruction tree evaluated by the effect resolver: every spell,
/// triggered and activated effect. Combinators express modes, iteration and
/// conditionals without per-card engine code.
//
// Fat payloads are boxed (`TokenDefinition` in every variant that carries
// one). An enum is sized to its largest variant, and at opt-level 0
// `run_effect`'s frame is roughly the sum of its arms' locals; unboxed it was
// 2.56 MB, which overflowed the 8 MB main stack three frames deep. `Box<T>`
// is serde-transparent and derefs at read sites, so keep new fat payloads
// boxed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    // ── Combinators ──────────────────────────────────────────────────────────
    /// Do nothing.
    Noop,
    /// Empty the resolution's `Selector::LastMoved` list, so a following
    /// "if that moved nothing" reads only the next move (Kynaios and Tiro's
    /// "each opponent who didn't put a land onto the battlefield").
    ClearLastMoved,
    /// Execute each inner effect in order.
    Seq(Vec<Effect>),
    /// If `cond` holds, execute `then`, else `else_`.
    If { cond: Predicate, then: Box<Effect>, else_: Box<Effect> },
    /// Execute `body` once per entity the `selector` resolves to.
    /// Inside `body`, `Selector::TriggerSource` refers to the current entity.
    ForEach { selector: Selector, body: Box<Effect> },
    /// Execute `body` `count` times.
    Repeat { count: Value, body: Box<Effect> },
    /// "Choose any number of [from]. [body for each]": the controller picks a
    /// subset (CR 608.2d), then `body` runs per pick bound as
    /// `Selector::TriggerSource`, like `ForEach` (Ripples of Potential). A
    /// headless seat takes what `headless_takes` matches (none if `None`).
    ForEachChosen {
        from: Selector,
        body: Box<Effect>,
        #[serde(default)]
        headless_takes: Option<SelectionRequirement>,
    },
    /// `body` once per `who` player, as controller, APNAP (CR 101.4) — "each
    /// opponent does X of their choice" (Rottenmouth Viper).
    EachPlayerDoes { who: PlayerRef, body: Box<Effect> },
    /// CR 705 — flip `count` coins (`Decision::CoinFlip`, heads = true), running
    /// `on_heads` or `on_tails` per flip. Karplusan Minotaur, Krark's Thumb.
    FlipCoin {
        count: Value,
        on_heads: Box<Effect>,
        on_tails: Box<Effect>,
    },
    /// CR 705 — "that player flips a coin" (Planar Chaos): the flip is made by
    /// `flipper` rather than the effect's controller, so their Krark's-Thumb
    /// advantage and their win/lose triggers apply.
    FlipCoinBy { flipper: PlayerRef, on_heads: Box<Effect>, on_tails: Box<Effect> },
    /// CR 705 — each `who` seat flips its own coin (APNAP), then `on_heads` /
    /// `on_tails` runs as that seat (Goblin Assassin).
    EachPlayerFlipsCoin { who: PlayerRef, on_heads: Box<Effect>, on_tails: Box<Effect> },
    /// CR 705 — flip until the controller loses or stops; a loss means zero
    /// wins. Then every `(threshold, effect)` with threshold ≤ wins runs, in
    /// order (Fiery Gambit).
    FlipCoinsUntilLoseOrStop { tiers: Vec<(u32, Box<Effect>)> },
    /// CR 705.2 — Mana Clash: you and `opponent` flip; each tails takes 1 from
    /// `source`; repeat until both heads.
    ManaClash { opponent: Selector },
    /// CR 706 — roll `count` d`sides` (`Decision::DieRoll`), running the first
    /// arm of `results` whose `[low, high]` covers each roll (none if no arm
    /// matches). The die analogue of `FlipCoin` (Goblin Goliath). CR 706.2
    /// modifiers via `modifier`, 706.2b rerolls via `reroll_at_most`; an
    /// engine-wide reroll granted by another source is still ⏳.
    RollDie {
        /// Number of sides on each die (e.g. 6 for d6, 20 for d20).
        /// Must be at least 2.
        sides: u8,
        /// Number of dice to roll. Each die rolls independently and
        /// runs its own results-table dispatch.
        count: Value,
        /// CR 706.2 — added to each natural roll before the table is read, floored at
        /// 1 but allowed past `sides` so an "N+" arm catches boosts. The decider
        /// still returns the natural roll. Default 0 for snapshot back-compat.
        #[serde(default = "crate::effect::zero_value")]
        modifier: Value,
        /// CR 706.2b — a natural result `<= reroll_at_most` is rerolled once and the
        /// new face kept; the modifier applies after. 0 (default, snapshot
        /// back-compat) never rerolls.
        #[serde(default)]
        reroll_at_most: u8,
        /// CR 706.6 — roll this many extra dice and drop that many lowest natural
        /// results before anything reads them (Berserker's Frenzy). Stacks with
        /// `RollExtraDiceIgnoreLowest` statics. Default 0 for snapshot back-compat.
        #[serde(default)]
        ignore_lowest: u8,
        /// CR 706.3a — results table of `(low, high, effect)`; the first arm with
        /// `low <= roll <= high` fires. `(low, sides, ..)` is "N+", `(n, n, ..)` one
        /// number.
        results: Vec<(u8, u8, Effect)>,
        /// CR 706.5 — with `count >= 2`, fires once after the per-die results if two
        /// natural faces match. Default `None` for snapshot back-compat.
        #[serde(default)]
        on_doubles: Option<Box<Effect>>,
    },
    /// CR 706 — roll two d`sides`, choose which result feeds `first` and which
    /// `second` (Wild Endeavor). One `DiceRolled` event, then it continues as
    /// [`Effect::AssignTwoDieResults`] so a suspended choice replays the faces.
    RollTwoDiceAssign { sides: u8, first: Box<Effect>, second: Box<Effect> },
    /// The choice half of [`Effect::RollTwoDiceAssign`], its faces fixed: the
    /// controller picks which of `a` / `b` `first` reads through
    /// `Value::LastDieRoll`; `second` reads the other. Equal faces ask nothing.
    AssignTwoDieResults { a: u8, b: u8, first: Box<Effect>, second: Box<Effect> },
    /// CR 706 — Chaos Dragon: every living seat rolls a d`sides` (one
    /// `DiceRolled` each, turn order from the controller); each opponent tied for
    /// the top is granted to the source as `Keyword::CantAttackPlayer` until end
    /// of combat.
    EachPlayerRollsSourceCantAttackHighest { sides: u8 },
    /// Modal — controller picks one of `modes` at cast time; the chosen index
    /// is stored in the stack item's `mode` field.
    ChooseMode(Vec<Effect>),
    /// "Choose `picks.len()` —": runs each mode index in `picks`, in order
    /// (Strixhaven Commands, Charms). CR 700.2d: no mode twice. The auto-decider
    /// supplies a per-card default; a mode-pick UI may override.
    ///
    /// Each target-bearing mode owns a slot by its position among the
    /// target-bearing modes in `picks` (first reads `ctx.targets[0]`, …), so
    /// "choose one or both" with different targets works (Steal the Show).
    /// Non-targeting modes see the full context. Cast-time validation
    /// (`target_filter_for_slot_in_mode`) uses the same default-`picks` order.
    ///
    /// Limitation: a decider picking another subset still supplies targets in
    /// default-picks slot order; full cast-time mode selection is in TODO.md.
    ///
    /// `up_to: Some(n)` is "choose up to N —" (CR 700.2): `n` is read at
    /// resolution and caps the picks (zero runs nothing); slots stay keyed to
    /// `picks`. The Ruinous Wrecking Crew (N = X).
    ChooseN {
        picks: Vec<u8>,
        modes: Vec<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        up_to: Option<Box<Value>>,
    },
    /// CR 700.2 — "choose up to N" with N a live count: `max`, capped at
    /// `modes.len()`, distinct modes (AutoDecider takes the first). Modes must
    /// not have target slots. Bumi, King of Three Trials.
    ChooseUpToN { max: Box<Value>, modes: Vec<Effect> },
    /// A resolution-time "choose one or more": `mode` always runs, a
    /// `Decision::ChooseModes` answer adds distinct modes, `cost` runs per added
    /// mode. Unused: printed escalate (CR 702.120a) is paid at cast via
    /// `CardDefinition::escalate` on a `ChooseModesCast` body.
    Escalate {
        modes: Vec<Effect>,
        cost: Box<Effect>,
    },
    /// CR 702.172 Spree: modes picked at cast (`GameAction::CastSpellSpree`),
    /// their costs added, stamped on `CardInstance.spree_modes`. They resolve in
    /// printed order, each target-bearing mode taking the next slot (`target`,
    /// then `additional_targets`); targets are validated at resolution.
    Spree { modes: Vec<SpreeMode> },
    /// FIN Tiered: exactly one mode, its extra cost paid at cast. Spree's cast
    /// plumbing with a one-mode validator (Fire/Ice/Thunder Magic).
    Tiered { modes: Vec<SpreeMode> },
    /// Cast-time multi-mode selection without per-mode costs: "choose one or
    /// both" (Choreographed Sparks: 1..2, no repeats), "choose up to four, same
    /// mode allowed" (Moment of Reckoning). Spree's cast and resolution plumbing,
    /// validated against min/max/repeats. A plain `CastSpell { mode }` runs that
    /// one mode. `max: 0` is "Choose X" (Doomsday Confluence).
    ChooseModesCast { modes: Vec<Effect>, min: u8, max: u8, allow_repeats: bool },
    /// BLB Seasons: "choose up to `budget` {P} worth of modes", repeats allowed;
    /// each mode costs `points[i]`. `ChooseModesCast`'s cast and resolution
    /// plumbing.
    ChooseModesByPoints { modes: Vec<Effect>, points: Vec<u8>, budget: u8 },
    /// "Choose one that hasn't been chosen —" (Captive Audience): picks an unused
    /// mode at resolution, recording it in `CardInstance.modes_chosen`; nothing
    /// once all are used.
    ChooseUnchosenMode { modes: Vec<Effect> },
    /// [`Effect::ChooseUnchosenMode`] whose record resets each turn (Teval's
    /// Judgment): `CardInstance.modes_chosen` carries a four-byte turn header,
    /// and a stale header clears the list on the next resolution.
    ChooseUnchosenModeThisTurn { modes: Vec<Effect> },
    /// "You may [body]": asks `ctx.controller` yes/no via
    /// `Decision::OptionalTrigger` (showing `description`) and runs `body` on
    /// `Bool(true)`. For a mana cost see `Effect::MayPay`.
    ///
    /// `AutoDecider` declines, as "you may" defaults to no. The bot is not the
    /// `AutoDecider`: `decide_pending_policy` takes pure-upside bodies and
    /// declines self-costs, so `MayDo` doesn't switch a good effect off.
    ///
    /// ⚠ The yes/no is asked before the body's targets: a test's
    /// `ScriptedDecider` needs `[Bool(true), Target(…)]`. The other order lands
    /// the `Target` on the prompt as "no" and silently skips the body.
    ///
    /// `description` is a `String` because `Effect` derives `Deserialize`.
    MayDo { description: String, body: Box<Effect> },
    /// [`Effect::MayDo`] routed to `who`: the yes/no is theirs and `body` runs
    /// with `who` as controller, target choice included (Ley Line).
    MayDoBy { who: PlayerRef, description: String, body: Box<Effect> },
    /// "[Player] may [A] or [B]": `who` picks one of `options` (a
    /// `ChooseOption` ask, judged by outcome for a bot) and its effect runs
    /// with `who` as controller. Headless, option 0 — list the do-nothing
    /// option first where there is one. Lavabrink Floodgates.
    PlayerChoosesOne { who: PlayerRef, options: Vec<VoteOption> },

    /// [`Effect::MayPay`] routed to `who`: the yes/no and payment are theirs, and
    /// `body` / `else_` run with them as controller (Phyrexian Tyranny).
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
    /// Pain's Reward life bidding: the controller opens, others may top it in
    /// turn order until a full pass; the high bidder loses the bid and `then`
    /// runs for them (CR 104.3 — bidding above your life is legal).
    LifeBidding { then: Box<Effect> },
    /// Plague of Vermin: from you, players may pay any life (CR 119.4) until a
    /// round with no payment; each makes a `definition` token per life paid.
    EachPlayerPaysLifeForTokens { definition: std::sync::Arc<crate::card::TokenDefinition> },
    /// "You may pay {X}. When you do, [body]": X via `Decision::ChooseAmount`
    /// (0 = decline, the AutoDecider default), capped by pool plus untapped
    /// sources; `body` runs with `ctx.x_value = X` (Tester of the Tangential).
    MayPayX { description: String, body: Box<Effect> },
    /// [`Effect::MayPayX`] with X at most `cap` ("X can't be greater than the
    /// amount of life you gained this turn" — Shanna, Purifying Blade).
    MayPayXUpTo { description: String, cap: Value, body: Box<Effect> },
    /// "You may pay {X}{X}" — [`Effect::MayPayX`] with the payment `times` ×
    /// X (Numa, Joraga Chieftain). Opaque to the target walkers: a targeted
    /// body goes inside an [`Effect::Reflexive`].
    MayPayXTimes { times: u32, description: String, body: Box<Effect> },
    /// "You may pay {X}{R}" — [`Effect::MayPayX`] plus a fixed `extra` part
    /// paid in the same payment, so X is sized to what's left after it
    /// (Flameblast Dragon).
    MayPayXPlus { extra: crate::mana::ManaCost, description: String, body: Box<Effect> },
    /// "You may pay any amount of {R}" — [`Effect::MayPayX`] whose X is paid
    /// in `color` pips rather than generic (Leyline Tyrant).
    MayPayXOfColor { color: crate::mana::Color, description: String, body: Box<Effect> },

    /// "You may pay [mana]. If you do, [body]": yes/no, then pay and run `body`;
    /// declined or unaffordable skips it. The paid sibling of `MayDo` (Bayou
    /// Groff).
    ///
    /// Payment takes the floating pool first, then untapped sources (CR 605.3a).
    /// An optional cost with its own X belongs in `MayPayX`.
    MayPay {
        description: String,
        mana_cost: crate::mana::ManaCost,
        body: Box<Effect>,
        /// Runs when the cost was declined or unpayable ("if you don't, …").
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// CR 119.4 — "you may pay N life. If you do, `body`", payable only at life ≥
    /// `amount`; declined or unpayable runs `else_`. Seymour Flux.
    MayPayLife {
        description: String,
        amount: Value,
        body: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "You may pay {X}, X ≤ `max`": X via `Decision::ChooseAmount` within
    /// `0..=min(max, pool)`, spent from the floating pool; `body` reads it as
    /// `Value::TriggerEventAmount` (Well of Lost Dreams). AutoDecider pays 0.
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

    /// CR 603.7 reflexive "when you do": wraps a targeted body whose targets are
    /// chosen after the gating cost is paid (`MayPay { body: Reflexive(..) }` —
    /// Itzquinth). Opaque to the cast/trigger-time target walk
    /// (`target_filter_for_slot`, `requires_target`); the body is auto-targeted
    /// fresh at resolution.
    Reflexive { body: Box<Effect> },

    /// "You may sacrifice [count] [filter]. If you do, [then]" (Bloodcrazed
    /// Socialite): offered only with a legal non-source candidate; the seat picks
    /// which (headless: weakest).
    MaySacrifice {
        description: String,
        filter: SelectionRequirement,
        count: Value,
        then: Box<Effect>,
        /// Runs if the cost was declined or no legal sacrifice existed.
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// CR 701.16 + 603.7 — "you may sacrifice this. If you do, [then]", else
    /// `else_`; offered only while the source is on the battlefield (Eden, Seat
    /// of the Sanctum).
    MaySacrificeSource {
        description: String,
        then: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "You may tap [count] untapped [filter] you control. If you do, [then]"
    /// (Caparocti Sunborn); offered with enough matches, taps the lowest-impact
    /// ones.
    MayTap {
        description: String,
        filter: SelectionRequirement,
        count: Value,
        then: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "You may discard [count] card(s). If you do, [then]", else `else_` (Toph,
    /// Hardheaded Teacher). Offered only with `count` cards in hand; discards the
    /// highest mana values.
    MayDiscard {
        description: String,
        count: Value,
        then: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "…, then discard `count` of them" (Casting of Bones): the controller
    /// discards from among the cards they drew earlier in this resolution.
    DiscardOneOfDrawn { count: Value },
    /// [`Effect::MayDiscard`] restricted to `filter` matches (offered only with
    /// `count` of them; highest mana values go). Drekavac: `then: Noop, else_:
    /// SacrificeSource`.
    MayDiscardMatching {
        description: String,
        count: Value,
        filter: SelectionRequirement,
        then: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// "You may reveal a [filter] card from your hand. If you do, `then`;
    /// otherwise `else_`" — the STX Snarl lands. Asked of `ctx.controller`,
    /// filter checked per hand card via `evaluate_requirement_on_card`.
    ///
    /// AutoDecider always reveals when it can (a bluffing decline isn't
    /// modeled). With no match, `else_` runs.
    IfRevealFromHand {
        filter: SelectionRequirement,
        then: Box<Effect>,
        else_: Box<Effect>,
    },

    // ── Damage / life ────────────────────────────────────────────────────────
    DealDamage { to: Selector, amount: Value },
    /// CR 702 Radiance — `amount` damage to `subject`'s creature and each other
    /// creature sharing a color with it (Cleansing Beam). `subject` carries slot
    /// 0, so targeting goes through the normal path; a colorless subject hits
    /// only itself.
    RadianceDamage { subject: Selector, amount: Value },
    /// `amount` to `subject`'s creature and each other creature with its name
    /// (Izzet Staticaster) — `RadianceDamage`'s shape.
    SameNameDamage { subject: Selector, amount: Value },
    /// Each creature you control deals `amount` to `to`, sourced from itself
    /// (Case of the Gateway Express).
    EachControlledCreatureDealsDamage { to: Selector, amount: Value },
    /// Damage to each player computed from what they control: `amount` × their
    /// `filter` permanents (Acidic Soil), or with `flat` a flat `amount` to each
    /// player with any (Disorder). No match, no damage.
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
    /// "N damage divided as you choose": targets chosen at cast across slots
    /// `0..max_targets` (each `filter`ed), split at resolution via
    /// `Decision::DivideDamage` (AutoDecider spreads evenly). Forked Bolt.
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
    /// Fireball: `DealDamageDivided`'s slots, each target taking `total / n`
    /// (remainder lost); no decision.
    DealDamageDividedEvenly {
        total: Value,
        filter: SelectionRequirement,
        max_targets: u8,
    },
    /// Two creatures fight: each deals damage equal to its power to the other at
    /// once. No-op when either side isn't a permanent (as printed). Chelonian
    /// Tackle.
    Fight { attacker: Selector, defender: Selector },
    /// Ezuri's Predation (CR 701.14): a `definition` token per `filter` creature,
    /// each fighting a different one; doubler extras fight nothing.
    CreateTokensToFightEach {
        filter: crate::card::SelectionRequirement,
        definition: std::sync::Arc<crate::card::TokenDefinition>,
    },
    /// One-sided fight: `source` deals its power to `target`, carrying `source`
    /// so lifelink / deathtouch / wither apply. No-op if either is missing (Tail
    /// Swipe).
    DealDamageEqualToPower { source: Selector, target: Selector },
    /// The permanent `source` resolves to (LKI once gone) deals `amount` to each
    /// `to` entity, carrying that creature's lifelink / deathtouch / infect
    /// (Saskia the Unyielding, where "it" isn't the ability's source).
    DealDamageFrom { source: Selector, to: Selector, amount: Value },
    /// `source` deals its power (read once) to each `targets` entity except
    /// itself, and with `each_opponent` to each of its controller's opponents;
    /// carries `source`. Chandra's Ignition. No-op without `source`.
    DealDamageEqualToPowerToEach {
        source: Selector,
        targets: Selector,
        each_opponent: bool,
    },
    /// Each `dealers` permanent deals its power to `target` (Nissa's Judgment),
    /// each carrying its dealer. Mirror of `DealDamageEqualToPowerToEach`.
    EachDealsDamageEqualToPower { dealers: Selector, target: Selector },
    /// Each `dealers` permanent deals `amount` to each `to` entity, each hit
    /// carrying its dealer (CR 120.3). Sarkhan the Masterless. Filtered sibling of
    /// `EachControlledCreatureDealsDamage`.
    EachDealsDamage { dealers: Selector, to: Selector, amount: Value },
    /// `first` and `second` damage each other at once, amounts read before either
    /// lands and SBAs waiting for both (CR 120.3, 704.3). Not a fight (CR
    /// 701.14): Grim Contest uses toughness. No-op if either is missing.
    DealDamageToEachOther { first: Selector, second: Selector, first_amount: Value, second_amount: Value },
    /// CR 701.12 — exchange control of the two permanents at once (Switcheroo);
    /// no-op if either is missing.
    ExchangeControl { a: Selector, b: Selector },
    /// Perplexing Chimera: exchange control of `what` and the triggering spell.
    /// The "choose new targets" rider isn't modeled.
    ExchangeControlWithTriggeringSpell { what: Selector },
    /// Twist Allegiance — you and each player `who` resolves to swap control
    /// of every creature you each control for `duration`; the swapped
    /// creatures untap and gain haste.
    ExchangeCreatureControlWith { who: Selector, duration: Duration },
    /// Overblaze — each resolved permanent's damage is doubled for the rest of
    /// the turn (CR 614.2), via `doubled_damage_sources_this_turn`.
    DoubleDamageFromSourceThisTurn { what: Selector },
    /// Jeska, Thrice Reborn's 0: until your next turn `what`'s combat damage to
    /// your opponents is tripled (CR 614.1a; `tripled_combat_damage_to_opponents`).
    TripleCombatDamageToYourOpponentsUntilYourNextTurn { what: Selector },
    /// Whims of the Fates: each player splits their permanents into `piles` piles
    /// (round-robin over a shuffle, no choice) and sacrifices one at random.
    EachPlayerSplitsAndSacrificesRandomPile { piles: u8 },
    /// "Separate [cards] into two piles. [chooser] chooses one": `splitter` makes
    /// the piles, `chooser` picks; the bodies read `Selector::SeparatedPile`.
    /// Invasion's split cards (Do or Die).
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
    /// Runs `body` with `what` as its targets — a resolution-time "choose" (not
    /// a target) feeding a body that reads targets (Canoptek Wraith, inside
    /// `ChooseOneAmong`).
    WithTargets { what: Selector, body: Box<Effect> },
    /// "For each different power among [filter], choose one" (Celestial
    /// Judgment), or with `up_to` "any number with different powers" (Sigardian
    /// Zealot); `chosen` / `other` run as for `ChooseSomeAmong`. Headless: own
    /// first, own best (mana value, toughness), theirs weakest; `up_to` own only.
    ChoosePerDistinctPower {
        filter: SelectionRequirement,
        up_to: bool,
        chosen: Box<Effect>,
        other: Box<Effect>,
    },
    /// Queen Kayla: one `what` pick per listed mana value, all asked before
    /// `chosen` moves them together; `up_to` lets a value pass.
    ChoosePerManaValue {
        what: Selector,
        values: Vec<u32>,
        up_to: bool,
        chosen: Box<Effect>,
        other: Box<Effect>,
    },
    /// `ChooseOneAmong` for `count` picks — exactly `count` (as many as there
    /// are), or with `up_to` any number to `count`. Haunting Voyage's "up to
    /// two", Insatiable Frugivore's three.
    ChooseSomeAmong {
        what: Selector,
        chooser: PlayerRef,
        count: Value,
        up_to: bool,
        chosen: Box<Effect>,
        other: Box<Effect>,
    },
    /// "Return a [`filter`] you control to hand", chosen at resolution, not
    /// targeted (Whitemane Lion ruling). `keep_best` returns the most valuable
    /// (Time Wipe); otherwise the cheapest, tapped first.
    ReturnOneYouControl { filter: SelectionRequirement, keep_best: bool },
    /// `ChooseOneAmong` with a uniform random pick (The Nipton Lottery).
    ChooseOneAtRandomAmong { what: Selector, chosen: Box<Effect>, other: Box<Effect> },
    /// "An opponent chooses one of [`what`]" as a veto: the opponent a wrapping
    /// `ChooseOpponentThen` named (else the most hostile) names one, and `then`
    /// runs on the rest (`SeparatedPile { chosen: false }`). Headless picks the
    /// highest mana value. Plargg and Nassari.
    OpponentVetoesOne { what: Selector, then: Box<Effect> },
    /// Abstract Performance: top `count` cards into a face-down pile, the next
    /// `count` face up; the hostile opponent picks one for your graveyard; you
    /// may cast a spell from the other free and take the rest. The chooser is
    /// asked before anything moves.
    FaceDownFaceUpPiles { count: u32 },
    /// CR 701.12 — exchange control of your `filter` permanent (chosen at
    /// resolution; non-UI: lowest CardId) and `with` (Vedalken Plotter).
    ExchangeControlChoosing { filter: SelectionRequirement, with: Selector },
    GainLife  { who: Selector, amount: Value },
    LoseLife  { who: Selector, amount: Value },
    /// CR 701.10d — each player gains their current life (Beacon of
    /// Immortality); 0 or less is unaffected.
    DoubleLife { who: Selector },
    /// Each resolved player loses half their own life (rounded per
    /// `rounded_up`) — per player, which `LoseLife` can't do. Stingerback
    /// Terror.
    LoseHalfLife { who: Selector, rounded_up: bool },
    /// "Each player loses `per` life for each [`filter`] they control"
    /// (Stronghold Discipline). Per-player evaluation, so each seat's own
    /// board sets its own loss.
    LoseLifePerControlled { who: Selector, filter: SelectionRequirement, per: Value },
    /// Each resolved player is dealt half their own life as damage (rounded per
    /// `rounded_up`), through the damage funnel unlike `LoseHalfLife`. Heartless
    /// Hidetsugu.
    DealHalfLifeDamage { to: Selector, rounded_up: bool },
    /// CR 119.5 — set a player's life total; they gain or lose the difference
    /// (`LifeGained` / `LifeLost`, nothing for zero). Biorhythm.
    SetLifeTotal { who: Selector, amount: Value },
    /// CR 701.12c — exchange the two players' life totals (Soul Conduit). Both
    /// totals are read first, then each gains or loses to the other's, firing
    /// life events. No-op on one player.
    ExchangeLifeTotals { a: Selector, b: Selector },
    /// "Can't gain life this turn" (Skullcrack): sets
    /// `Player.cannot_gain_life_this_turn`, cleared by `do_untap`. The sourceless
    /// one-shot counterpart of `StaticEffect::PlayerCannotGainLife`.
    LifeGainLockThisTurn { who: Selector },
    /// One-turn "[selected players]' life total can't change this turn" lock —
    /// both gain and loss are dropped. Sets `Player.life_locked_this_turn`,
    /// cleared by `do_untap` at the turn boundary. Flare of Fortitude.
    LifeLockThisTurn { who: Selector },
    /// "Players gain hexproof until end of turn" (Everybody Lives!). Sets
    /// `Player.hexproof_this_turn`, cleared at the next untap.
    PlayerHexproofThisTurn { who: Selector },
    /// "Players can't lose life this turn" (Everybody Lives!) — unlike
    /// `LifeLockThisTurn`, life can still be gained.
    CantLoseLifeThisTurn { who: Selector },
    /// CR 104.3d — Angel's Grace: this turn the controller can't lose and
    /// opponents can't win; with `damage_floor`, damage stops at 1 life. Cleared
    /// by `do_untap`.
    CantLoseThisTurn { damage_floor: bool },
    /// CR 614 / 104.3 — "The next time you would lose the game this turn,
    /// instead draw seven cards and your life total becomes 1" (Stunning
    /// Reversal): arms one turn-scoped loss replacement for the controller.
    NextLossThisTurnDrawSevenAtOne,
    /// Forbidding Spirit: until your next turn, attacking you or your
    /// planeswalkers costs {amount} per creature
    /// (`Player.attack_tax_until_your_turn`).
    TaxAttackersUntilYourNextTurn { amount: Value },
    /// "Until your next turn, creatures can't attack you" (Chronomantic
    /// Escape): the controller's `attackable_only_by_until_next_turn` shield,
    /// admitting no attacker. Planeswalkers they control stay attackable.
    CreaturesCantAttackYouUntilYourNextTurn,
    /// Sivitri, Dragon Master's +1: the same tax in life
    /// (`Player.attack_life_tax_until_your_turn`).
    TaxAttackersInLifeUntilYourNextTurn { amount: Value },
    /// Channel: this turn, pay 1 life per {C} of shortfall when paying costs
    /// (`Player.channel_life_for_mana`).
    ChannelLifeForMana,
    /// "Can't gain life for the rest of the game" (Screaming Nemesis): sets
    /// `Player.cannot_gain_life` permanently. Non-players are ignored, so
    /// `Target(0)` over a damage target fires only on a player.
    LifeGainLockGame { who: Selector },
    /// Set `Player.spells_uncounterable_this_turn` on each resolved player —
    /// "spells you control can't be countered for the rest of this turn"
    /// (Veil of Summer). Cleared at the next untap.
    GrantSpellsUncounterableThisTurn { who: Selector },
    /// Like `GrantSpellsUncounterableThisTurn` but only for *creature* spells
    /// (Domri, Anarch of Bolas's "Creature spells you cast this turn can't be
    /// countered"). Cleared at the next untap.
    GrantCreatureSpellsUncounterableThisTurn { who: Selector },
    /// Veil of Summer: adds `colors` to each player's
    /// `hexproof_from_colors_this_turn`; cleared at the next untap.
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
    /// Hope of Ghirapur: `CantCastNoncreatureThisTurn` re-armed until the
    /// controller's next turn (`Player.noncreature_locked_by`).
    CantCastNoncreatureUntilYourNextTurn { who: Selector },
    /// "Each player names a card" — one `NameCard` decision per resolved seat
    /// in APNAP order, stashed in `GameState.scratch.names_this_resolution` for a
    /// later step in the same resolution to read (Conundrum Sphinx).
    EachPlayerNamesCard { who: PlayerRef },
    /// Conundrum Sphinx: each player reveals their top card; the named one goes
    /// to hand, otherwise bottom (names from `EachPlayerNamesCard`).
    EachPlayerRevealTopKeepIfNamed { who: PlayerRef },
    /// Controller loses `amount` life, a different selector gains it.
    Drain { from: Selector, to: Selector, amount: Value },
    /// "Each opponent loses N. You gain life equal to the life lost this way"
    /// (Gray Merchant): `to` gains the total actually lost after replacement (CR
    /// 119.3). `Drain` gains `amount` once — one opponent's worth at a table.
    DrainLifeLost { from: Selector, to: Selector, amount: Value },

    /// CR 122 / 107.16 — the controller gets `amount` energy counters
    /// ({E}). Energy is a per-player resource pool (`Player.energy`), not
    /// tied to any object. "You get {E}{E}" → `AddEnergy(Const(2))`.
    AddEnergy(Value),
    /// The controller gets `amount` experience counters (`Player.experience`).
    /// A per-player resource that only accumulates; payoffs read the count.
    /// Mizzix ("get an experience counter") → `AddExperience(Const(1))`.
    AddExperience(Value),
    /// Pay `amount` energy at resolution, then `then`; nothing if short. Models
    /// "Pay {E}{E}{E}: …" without an `ActivatedAbility` cost field.
    PayEnergy { amount: u32, then: Box<Effect> },
    /// "You may pay {E} equal to `amount`; if you do, `then`" — `PayEnergy` with
    /// a `Value` (Jolted Awake). Always paid when affordable.
    PayEnergyValue { amount: Value, then: Box<Effect> },
    /// CR 702.62 — remove a time counter from the resolving ability's source
    /// if it is a suspended card in exile; removing the last casts it through
    /// the suspend funnel. Nihilith's graveyard accelerant.
    RemoveTimeCounterFromSuspendedSource,
    /// CR 701.56 — `who` time travels, adding or removing a time counter on each
    /// permanent / suspended card with one. Bot heuristic: remove from suspended
    /// cards, add to permanents. Per-object UI choice is a follow-up.
    TimeTravel { who: PlayerRef },
    /// Dust of Moments: remove (`add: false`) or add `amount` time counters on
    /// each permanent with one and each suspended card. Removing the last casts
    /// it (CR 702.62).
    TimeCountersOnEachPermanentAndSuspended { add: bool, amount: u32 },
    /// "Pay any amount of {E}; deal that much damage" (Galvanic Discharge). Bot
    /// heuristic: exact lethal to a creature (capped by energy), everything at a
    /// walker or player. UI prompting is a follow-up.
    PayAnyEnergyDealDamage { to: Selector },
    /// "You may pay any amount of {E}, then `then`" (Aether Spike): the amount
    /// via `Decision::ChooseAmount`, stashed in
    /// `state.energy_paid_this_resolution` for `Value::EnergyPaidThisEffect`.
    /// Bots pay everything.
    PayAnyEnergy { then: Box<Effect> },
    /// Each player may exile their hand and draw `amount`; at `amount` ≥
    /// `play_if_at_least` you may play your exiled cards until the end of your
    /// next turn (Wheel of Potential). Headless takes it if its hand is smaller.
    EachPlayerMayExileHandDraw { amount: Value, play_if_at_least: u32 },
    /// Inniaz, the Gale Force: the controller picks one `filter` permanent from
    /// each living seat's right neighbour (CR 101.4, the previous seat); all
    /// change control at once, for good.
    EachPlayerGainsChosenFromRightNeighbor { filter: SelectionRequirement },
    /// CR 107.16 — "sacrifice / return this unless you pay {E}": pays if able,
    /// else `otherwise` (Lathnu Hellion).
    PayEnergyOrElse { amount: u32, otherwise: Box<Effect> },
    /// `PayEnergyOrElse` with a `Value` amount read at resolution (Volatile
    /// Stormdrake).
    PayEnergyOrElseValue { amount: Value, otherwise: Box<Effect> },
    /// "Sacrifice this unless you pay [mana]" (Archway Commons): pays from the
    /// floating pool if it can, else runs `otherwise`. The mana sibling of
    /// `PayEnergyOrElse`.
    PayManaOrElse { mana_cost: crate::mana::ManaCost, otherwise: Box<Effect> },
    /// CR 702.29 — echo's resolution half for a `wants_ui` controller ("sacrifice
    /// unless you pay"); `None` = discard echo. Mana echo auto-taps like
    /// `process_echo`.
    EchoPayOrSacrifice { mana_cost: Option<crate::mana::ManaCost> },
    /// CR 702.24 — the `wants_ui` sibling for cumulative upkeep: "sacrifice
    /// this unless you pay `cost` × its age counters". Mana/Life kinds only;
    /// the age counter was already added by `process_cumulative_upkeep`.
    CumulativeUpkeepPayOrSacrifice { cost: crate::card::CumulativeUpkeepCost },
    /// Balance: players sacrifice lands / creatures to the fewest, discard to the
    /// smallest hand, keeping the priciest (then strongest). A `wants_ui` picker
    /// is a follow-up.
    Balance,
    /// Generalized Balance: per `filters` entry, each player sacrifices down to
    /// the fewest matches anyone controls (keeping highest mana value, then
    /// power); with `hands`, hands discard down too. Balance is `[Land,
    /// Creature] + hands`; Balancing Act is `[Permanent]`.
    BalanceMatching { filters: Vec<SelectionRequirement>, hands: bool },
    /// Genesis Wave: reveal the top X; permanents with mana value ≤ X enter, the
    /// rest go to the graveyard ("any number" taken as all).
    GenesisWave,
    /// CR 107.16 — Amped Raptor: exile the top card, then may pay `energy` {E} to
    /// cast it free (it stays exiled otherwise).
    ExileTopMayPayEnergyToCast { energy: u32 },

    // ── Cards / draw / discard / mill ────────────────────────────────────────
    Draw    { who: Selector, amount: Value },
    /// CR 701.48 Learn — `who` may fetch a Lesson from their sideboard or rummage
    /// (`Decision::Learn`). With no sideboard Lesson it draws one card, as before.
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
    /// "`who` reveals `count` cards from their hand. You choose one of those
    /// cards. That player exiles it" (Taster of Wares): the revealer picks the
    /// cards shown, the controller the one exiled (`Selector::LastMoved`).
    RevealFromHandExileChosen { who: PlayerRef, count: Value },
    /// `ExileFromHand` whose cards are linked to the source (`exiled_with`),
    /// for "exile a card from your hand" on a permanent that later counts
    /// what it exiled (Azor's Gateway).
    ExileFromHandLinked { who: Selector, amount: Value },
    /// "Exile a card from your hand face down. It becomes foretold" with cost
    /// reduced by {`reduce`} (Ethereal Valkyrie). Not a foretell action (Ranar's
    /// free one is untouched); not castable this turn (CR 702.143b).
    ForetellFromHand { reduce: u32 },
    /// Arcane Artisan: each resolved player exiles a card from hand (bot: best
    /// creature); a creature yields a token copy stamped as created by the source
    /// (`Selector::TokensCreatedBySource`).
    ExileFromHandCopyCreature { who: Selector },
    /// Each player discards their hand and draws as many as it held — read
    /// before discarding, which `Discard` + `Draw` can't (Soratami Seer).
    DiscardHandDrawThatMany { who: Selector },
    /// [`Self::DiscardHandDrawThatMany`] drawing `less` fewer — "then draws
    /// that many cards minus one" (Dark Deal).
    DiscardHandDrawThatManyLess { who: Selector, less: u32 },
    /// Wrench Mind: `who` discards `count` unless they discard an `instead` card
    /// (auto: lowest-mana-value match).
    DiscardUnlessKind { who: PlayerRef, count: Value, instead: SelectionRequirement },
    /// Ad Nauseam: repeatedly offer the top card to hand for its mana value in
    /// life, until declined or empty. AutoDecider declines at once.
    RevealTopToHandLoseLifeRepeat,
    /// Discard any number of cards, adding to
    /// `state.cards_discarded_this_resolution` for a following
    /// `Value::CardsDiscardedThisEffect` (Colossus of the Blood Age). AutoDecider
    /// discards none; `DecisionAnswer::Discard` scripts a list.
    DiscardAnyNumber {
        who: Selector,
        /// Restricts which hand cards may be picked ("discard any number of
        /// *creature* cards" — Mind Maggots). `Any` for the unrestricted
        /// wording.
        #[serde(default = "SelectionRequirement::any")]
        filter: SelectionRequirement,
        /// The printed cap on "discard up to N" (Greasewrench Goblin); `None` is "any
        /// number". Enforced on the answer, so an over-long UI pick is truncated.
        #[serde(default)]
        max: Option<Value>,
    },
    /// "No maximum hand size for the rest of the game" (Wisdom of Ages): sets
    /// `Player.max_hand_size = None`, skipping the CR 514.1 discard in
    /// `do_cleanup`.
    SetNoMaxHandSize { who: Selector },
    /// `who` chooses a card from their hand and puts it on top of their library
    /// (CR 701). No-op if the hand is empty. Enter the Infinite's "then put a
    /// card from your hand on top of your library."
    PutCardFromHandOnTopOfLibrary { who: Selector },
    /// "Put `count` cards from your hand on the bottom" (Sawtooth Loon), after a
    /// `Draw`. Auto-picked like [`Effect::PutCardFromHandOnTopOfLibrary`]; a UI
    /// seat isn't prompted.
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
    /// "For each player, choose a [`keep`] that player controls. Then destroy
    /// all [`of`] except those chosen" (Unstable Glyphbridge) — the
    /// controller makes every pick.
    ChooseOnePerPlayerDestroyRest { keep: SelectionRequirement, of: SelectionRequirement },
    /// "Each player chooses any number of creatures they control with total
    /// power `max_power` or less, then sacrifices all other creatures they
    /// control" (Destined Confrontation).
    EachPlayerKeepsCreaturesTotalPower { max_power: i32 },
    /// "Each player chooses `keep` permanents they control, then sacrifices the
    /// rest" (Razia's Purification). Each player's own pick — the auto picker
    /// keeps their highest-mana-value permanents.
    EachPlayerKeepsNSacrificesRest {
        keep: Value,
        /// Narrows both the keep pool and the sacrifice (Natural Balance is
        /// lands-only). `None` = every permanent (Razia's Purification).
        #[serde(default)]
        filter: Option<SelectionRequirement>,
        /// "Destroy all other [filter] permanents" instead of sacrificing
        /// them (Fall of the First Civilization's III): indestructible and
        /// regeneration apply (CR 701.8), and the rest are destroyed at once.
        #[serde(default)]
        destroy: bool,
    },
    /// Curfew: each player (APNAP) returns one of their `filter` permanents to
    /// hand. `opponents`: only opponents (Summon: Valefor).
    EachPlayerReturnsAMatchingPermanent {
        filter: SelectionRequirement,
        #[serde(default)]
        opponents: bool,
    },
    /// CR 402.2b — sets `Player.max_hand_size = Some(size)` (Null Profusion);
    /// cleanup (CR 514.1) discards down to it.
    SetMaxHandSize { who: Selector, size: Value },
    /// CR 402.2 — "your maximum hand size is reduced by `by`" (for the rest
    /// of the game): a player with no maximum hand size stays without one.
    /// Inspired Idea's uncleaved half.
    ReduceMaxHandSize { who: Selector, by: Value },
    Mill    { who: Selector, amount: Value },
    /// Mill `amount`, then one `filter` card among them to hand (Cache Grab).
    MillThenToHand {
        amount: Value,
        filter: SelectionRequirement,
        /// Runs when no milled card matched (Patient Naturalist's "if you
        /// can't, create a Treasure").
        #[serde(default)]
        otherwise: Option<Box<Effect>>,
    },
    /// [`MillThenToHand`] taking up to `take` matches, a [`Value`] so it can scale
    /// (Gather the Pack's spell mastery).
    MillThenToHandN {
        amount: Value,
        filter: SelectionRequirement,
        take: Value,
        /// Runs when no card was taken (Nashi's "if you put no cards into
        /// your hand this way, put a +1/+1 counter on Nashi").
        #[serde(default)]
        otherwise: Option<Box<Effect>>,
    },
    /// Mill one, then run `land`, `creature` or `noncreature` by its type (Old
    /// Rutstein). The branches don't reference the card.
    MillThenBranchByType {
        land: Box<Effect>,
        creature: Box<Effect>,
        noncreature: Box<Effect>,
    },
    /// `who` mills `amount`; the controller draws per milled `filter` card
    /// (Coerced Confession).
    MillThenDrawPerType { who: Selector, amount: Value, filter: SelectionRequirement },
    /// Reveal cards from the top of each resolved player's library until
    /// `lands` land cards are revealed, then put all revealed cards into
    /// that player's graveyard (Mind Grind, Consuming Aberration's trigger).
    MillUntilLands { who: Selector, lands: Value },
    /// CR 701.13 — each resolved player mills one card at a time until a card
    /// matching `filter` or `max` cards have been milled, whichever comes
    /// first; the milled cards are `Selector::LastMoved` (Helm of Obedience).
    MillUntilMatching { who: Selector, filter: SelectionRequirement, max: Value },
    /// CR 701.13 — `who` mills two, repeating while they share `axis`. Sphinx's
    /// Tutelage; The Tale of Tamiyo draws per repeat.
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
    /// "Exile that many cards from the bottom of your library" — the
    /// controller's, each stamped `exiled_with = source` (The Temporal Anchor).
    ExileBottomOfLibraryLinked { amount: Value },
    /// Dance with Calamity: repeatedly may exile the top card; if the total mana
    /// value is `limit` or less, `then`. Headless keeps going below `stop_at`.
    /// Cards are recorded for `Selector::ExiledThisResolution`.
    ExileTopPushingLuck { stop_at: u32, limit: u32, then: Box<Effect> },
    /// "If a source you control would deal noncombat damage to a permanent or
    /// player this turn, it deals that much damage plus `amount` instead"
    /// (Taii Wakeen, Perfect Shot). Turn-scoped, stacking across activations.
    YourNoncombatDamageBonusThisTurn { amount: Value },
    /// CR 702.170 — the resolving spell is exiled plotted instead of the
    /// graveyard (Lilah, Undefeated Slickshot).
    PlotSpellOnResolve { what: Selector },
    /// The spell `what` names is exiled with a `counter` counter as it resolves
    /// (Goliath Daydreamer); not if it doesn't resolve.
    ExileSpellOnResolveWithCounter { what: Selector, counter: crate::card::CounterType },
    /// CR 702.62a — the spell `what` names is exiled with `time_counters` as it
    /// resolves and gains suspend (Gandalf of the Secret Fire).
    SuspendSpellOnResolve { what: Selector, time_counters: u8 },
    /// "You may cast a spell from among [filter] cards in exile free" during the
    /// resolution, timing ignored (Goliath Daydreamer).
    MayCastFromExileFree { filter: SelectionRequirement },
    /// Invoke Calamity: cast up to `count` `filter` spells with total mana value
    /// ≤ `total_mana_value` free from graveyard and/or hand, exiled instead of
    /// going to the graveyard. Cast in sequence (the second resolves first).
    CastUpToFreeFromHandAndGraveyard { count: u32, filter: SelectionRequirement, total_mana_value: u32 },
    /// "You may cast a [filter] spell from your hand, from your graveyard, or
    /// from among cards you own in exile without paying its mana cost" (Kaya
    /// the Inexorable's emblem). One pick; face-down exiles are excluded.
    CastOneFreeFromHandGraveyardOrExile { filter: SelectionRequirement },
    /// "You may cast a permanent spell with mana value `max_mv` or less from
    /// your hand without paying its mana cost. If you don't, `else_`"
    /// (Kellan, the Kid).
    MayCastPermanentFromHandFree { max_mv: Value, else_: Box<Effect> },
    /// "You may cast a [filter] spell with mana value ≤ `max_mv` from hand free;
    /// if you don't, `else_`" (Baral and Kari Zev).
    MayCastFromHandFreeMatching { filter: SelectionRequirement, max_mv: Value, else_: Box<Effect> },
    /// "[`who`] may cast a spell from hand free. If they do, `if_cast`, else
    /// `else_`" (Cmdr. Troi). Headless casts the priciest; `if_cast` runs as the
    /// controller with `Target(0)` = the cast spell.
    PlayerMayCastFromHandFree { who: PlayerRef, if_cast: Box<Effect>, else_: Box<Effect> },
    /// Assimilate (Star Trek) — each card `what` names enters under the
    /// controller's control (a permanent: its control is gained) with a +1/+1
    /// counter, an artifact creature whose only creature type is Borg.
    Assimilate { what: Selector },
    /// "Reveal until a creature card, assimilate it, the rest on the bottom at
    /// random" (The Borg Queen); `one_of_all`: every `who` reveals and only one
    /// card is assimilated (Gift of Perfection).
    RevealUntilCreatureAssimilate { who: PlayerRef, one_of_all: bool },
    /// Process — "you may put up to `count` cards an opponent owns from exile
    /// into their graveyard. If you do, [`then`]." On yes the oldest eligible
    /// cards move; none eligible or declined skips `then`, which reads the
    /// trigger's targets (Wasteland Strangler).
    Process { count: u32, then: Box<Effect> },
    /// Each resolved player mills half their own library (rounded per
    /// `rounded_up`), which `Mill` can't scale per player. Lord Xander.
    MillHalf { who: Selector, rounded_up: bool },
    /// Each player discards half their own hand (rounded per `rounded_up`), picked
    /// as `Discard` does. Lord Xander.
    DiscardHalf { who: Selector, rounded_up: bool },
    /// Each player sacrifices half their own `filter` permanents (rounded per
    /// `rounded_up`). Lord Xander.
    SacrificeHalf { who: Selector, filter: SelectionRequirement, rounded_up: bool },
    /// "[Each player] chooses up to `keep` [filter] they control, then
    /// sacrifices the rest" (Archfiend of Depravity). Each keeps its best
    /// (the inverse of `Sacrifice`'s weakest-first pick).
    SacrificeAllButN { who: Selector, keep: Value, filter: SelectionRequirement },
    /// Killing Wave: APNAP, each player chooses which creatures to pay `amount`
    /// life for (within their life); the rest are sacrificed together.
    EachCreatureSacrificedUnlessLife { amount: Value },
    /// "Each opponent chooses a [filter] card in their graveyard. Put those
    /// cards [to]" (Dredge the Mire) — each chooser gives up its least
    /// valuable match.
    EachOpponentChoosesFromGraveyard { filter: SelectionRequirement, to: ZoneDest },
    /// "Target opponent puts a [filter] card of their choice from their
    /// graveyard [to]" (Visions of Dread): `EachOpponentChoosesFromGraveyard`
    /// with the choosing seats named by `who`.
    PlayerChoosesFromGraveyard { who: PlayerRef, filter: SelectionRequirement, to: ZoneDest },
    /// "Each other player may draw a card. Whenever a card is drawn this way,
    /// [per_draw]" (Explosion of Riches) — `per_draw` runs under the
    /// resolving controller once per accepted draw.
    EachOtherPlayerMayDraw { per_draw: Box<Effect> },
    /// "Each player who discarded a card with the greatest mana value among
    /// cards discarded this way loses life equal to that mana value" (Scythe
    /// Specter). Reads this resolution's discards.
    GreatestDiscardersLoseLife,
    /// Archangel of Strife: each seat's war (0) / peace (1) pick is stamped on
    /// `modes_chosen` for `StaticEffect::WarOrPeace`. Bots: war for the
    /// controller, peace for others.
    EachPlayerChoosesWarOrPeace,
    /// "Each opponent sacrifices a permanent of their choice that shares a
    /// card type with [what]" (Martyr's Bond) — `what` read through LKI.
    EachOpponentSacrificesSharingTypeWith { what: Selector },
    Scry    { who: PlayerRef, amount: Value },
    Surveil { who: PlayerRef, amount: Value },
    LookAtTop { who: PlayerRef, amount: Value },
    /// Look at the top `amount` cards of `who`'s library and put them back in
    /// any order — all stay on top, none bottomed (Index, Spire Owl, Sage
    /// Owl). Distinct from Scry, which may bottom cards.
    RearrangeTop { who: PlayerRef, amount: Value },
    /// "…and the rest on the bottom of your library in any order": `who`
    /// orders the bottom `count` cards of their library (the ones an effect
    /// just bottomed). Runs after the effect that bottomed them.
    OrderLibraryBottom { who: PlayerRef, count: Value },
    /// Look at the top `count`; reveal up to `take` `filter` cards and put them
    /// on top, the rest on the bottom in any order. Fertile Thicket, Munda.
    LookTopKeepMatchingOnTop {
        who: PlayerRef,
        count: Value,
        take: Value,
        filter: SelectionRequirement,
    },
    /// CR 701.37 — *monstrosity N*. If the source isn't already monstrous,
    /// put N +1/+1 counters on it and it becomes monstrous (emitting
    /// `GameEvent::BecameMonstrous`). Once monstrous, this is a no-op.
    Monstrosity { n: Value },
    /// CR 701.15 — goad each `what` creature: the controller joins its
    /// `goaded_by`, so it must attack each combat, a non-goader if able, until
    /// that goader's next turn. Disrupt Decorum.
    Goad { what: Selector },
    /// CR 701.15a — "[what] is no longer goaded" (Serene Sleuth): ends every
    /// resolved and held goad on each creature. A static that says a creature
    /// "is goaded" (an attached Aura) keeps applying.
    Ungoad { what: Selector },
    /// "…is goaded for the rest of the game" (Nettling Nuisance): `Goad`, and
    /// the goad never expires.
    GoadForTheGame { what: Selector },
    /// Radiant Performer (CR 707.10): copy the spell `what` once per other
    /// permanent or player it could target, each copy (yours) aimed at a
    /// different one. Spells only.
    CopySpellForEachOtherLegalTarget { what: Selector },
    /// Precursor Golem — CR 707.10: the spell's CASTER copies the spell `what`
    /// names once for each other permanent matching `filter` it could target,
    /// each copy aimed at a different one.
    CasterCopiesSpellForEachOtherMatchingTarget { what: Selector, filter: crate::card::SelectionRequirement },
    /// CR 701.15 / 611.2b — `Goad` that lasts as long as `hold` does rather
    /// than until the goader's next turn (Hot Pursuit, Immortal Obligation).
    GoadWhile { what: Selector, hold: GoadLasts },
    /// Territorial Hellkite: remember a random opponent the source didn't attack
    /// last combat as its `chosen_player` (`MustAttackChosenPlayer`); with none,
    /// forget it and tap the source.
    ChooseRandomOpponentNotAttackedLastCombat,
    /// "You may attach to it any number of [what]" — the controller chooses
    /// which of the permanents `what` resolves to become attached to `to`
    /// (Bruna, Light of Alabaster; Heavenly Blademaster).
    AttachAnyNumberTo { what: Selector, to: Selector },
    /// "For each [what], you may attach it to a creature you control"
    /// (Inventory Management): each one's host is its own optional pick,
    /// asked in turn before any moves (headless: your greatest-power one).
    AttachEachToCreatureYouControl { what: Selector },
    /// "For each of [to], you may attach a [what] to it" (Goldwardens'
    /// Gambit): one optional pick per host, each attachment chosen at most
    /// once, all moved together after the picks (2023-02-04 ruling).
    AttachDistinctToEach { what: Selector, to: Selector },
    /// "Exile all other spells and counter all abilities" (Summary
    /// Dismissal). Uncounterable spells are exiled too; copies cease to exist.
    ExileAllOtherSpellsCounterAllAbilities,
    /// "Double the number of each kind of counter you have" (Aetheric
    /// Amplifier): `who`'s energy, experience and poison counters.
    DoublePlayerCounters { who: PlayerRef },
    /// "Reverse the game's turn order" (Aeon Engine). A second reversal
    /// restores the original order.
    ReverseTurnOrder,
    /// Wakanda Forever!: reveal the top `count`; one permanent card onto the
    /// battlefield (with an indestructible counter if set), one into hand, the
    /// rest to the graveyard. The engine picks the two priciest permanents.
    RevealDeployOneTakeOne { count: Value, indestructible: bool },
    /// "Choose one at random —" (Umaro, Raging Yeti). Not a die roll. The
    /// mode is picked as it resolves, and a targeted mode takes the
    /// auto-picker's target then.
    ChooseModeAtRandom(Vec<Effect>),
    /// "Starting with the next opponent in turn order, each opponent chooses
    /// a [filter] card in your graveyard that hasn't been chosen. Return each
    /// card chosen this way to the battlefield under your control" (Rejoin
    /// the Fight).
    EachOpponentReturnsFromYourGraveyard { filter: crate::card::SelectionRequirement },
    /// Free-cast rider (Strago and Relm): the creature spells `what` names gain
    /// haste and/or end-step sacrifice as they resolve. Run after the cast.
    GrantCastSpellRiders { what: Selector, haste: bool, sacrifice_eot: bool },
    /// CR 702.88 — the spell(s) `what` names, on the stack, gain rebound
    /// (Ojer Pakpatiq, Deepest Epoch: "whenever you cast an instant spell
    /// from your hand, it gains rebound").
    GrantReboundToSpell { what: Selector },
    /// "You may cast any number of spells from among [what] without paying
    /// their mana costs. Then each player who owns a spell you cast this way
    /// loses life equal to its mana value" (Kefka, Dancing Mad).
    CastExiledFreeOwnersLoseLife { what: Selector },
    /// "You may cast any number of [nonland] cards exiled this way without
    /// paying their mana costs" (Etali, Primal Storm) — Kefka's loop without
    /// the owners' life loss. Lands are skipped.
    CastExiledFree { what: Selector },
    /// CR 603.7 — Lucius the Eternal: when `what` leaves the battlefield, return
    /// this card from exile if it's still there.
    ReturnSourceWhenTargetLeaves { what: Selector },
    /// "For each opponent, choose a [filter] that player controls. Destroy the
    /// chosen permanents" (Ultimate Magic: Meteor). `random_one`: only one pick,
    /// chosen by the game RNG, is destroyed (Chaos Defiler).
    DestroyOnePerOpponent {
        filter: crate::card::SelectionRequirement,
        #[serde(default)]
        random_one: bool,
    },
    /// "For each kind of counter among `who`'s `filter` permanents, put one of
    /// that kind on [`onto`]" — keyword counters included (CR 122.1b); several
    /// recipients take kinds in turn (Exotic Pets). `or_plus_one` (Bribe Taker):
    /// only shield and keyword kinds are kept, the rest become +1/+1.
    AddCounterOfEachKindAmong { onto: Selector, who: PlayerRef, filter: crate::card::SelectionRequirement, or_plus_one: bool },
    /// "For each kind of counter on target permanent or player, give that
    /// permanent or player another counter of that kind" (Skyship Plunderer).
    AddCounterOfEachKindOn { what: Selector },
    /// Dramatist's Puppet (CR 122.1): per counter kind on the target, add or
    /// remove one — more helpful kinds on your side, fewer on theirs.
    AddOrRemoveCounterOfEachKindOn { what: Selector },
    /// Aven Courier: copy a counter kind from your permanent onto `what` if it
    /// lacks it; picks keyword, then shield, then +1/+1, never harmful.
    AddMissingCounterKindFromYours { what: Selector },
    /// Contractual Safeguard: a counter kind on your creature goes onto each
    /// other creature you control. `shield_first`: a shield counter on your best
    /// creature first, which may be the kind chosen.
    SpreadCounterKindToOthers { shield_first: bool },
    /// "Put the same number of each kind of counter on [`to`]" as [`from`]
    /// has, keyword counters included (Denry Klin, Editor in Chief).
    CopyCountersOnto { from: Selector, to: Selector },
    /// "Exile target creature and all other creatures its controller
    /// controls with the same name. That player investigates for each
    /// nontoken creature exiled this way" (Declaration in Stone).
    ExileSameNameInvestigate { what: Selector },
    /// "Each player gains control of all nontoken permanents they own"
    /// (Brooding Saurian). Ends every theft of a nontoken permanent, the
    /// temporary ones included.
    OwnersGainControlOfNontokens,
    /// "Each player gains control of all [filter] they own" (Alicia Masters:
    /// creatures, tokens included).
    OwnersGainControlOf { filter: SelectionRequirement },
    /// "Choose a color. [This] deals `amount` damage to each creature of the
    /// chosen color" (Sudden Demise). The engine picks the color that kills
    /// the most opposing value net of the caster's own.
    DamageEachCreatureOfChosenColor { amount: Value },
    /// CR 701.12g — "exchange its power and the power of `b`" (Serene
    /// Master): each gets the other's current power as a layer-7b set for
    /// `duration`. Nothing happens unless both are creatures on the battlefield.
    ExchangePower { a: Selector, b: Selector, duration: Duration },
    /// Order of Succession (CR 101.4): pick a direction; each player takes a
    /// creature of the next player that way. The caster picks the direction that
    /// nets it most; each chooser takes the next player's best.
    EachPlayerTakesCreatureOfNext,
    /// Lim-Dûl's Vault — look at the top five; as many times as you choose,
    /// pay 1 life, bottom them and look at the next five; then shuffle and put
    /// the last five on top. The controller is asked before each dig.
    LookTopFiveDigForLife,
    /// One `body` trigger per `what` permanent, sourced from it (`Selector::This`)
    /// with targets picked as it triggers — the token-copy "except it has 'When
    /// this enters, …'" rider (Aggressive Biomancy), whose enters event a granted
    /// ability would already have missed.
    EachPushesTrigger { what: Selector, body: Box<Effect> },
    /// CR 700.8 — "Each player chooses a party from among creatures they
    /// control, then sacrifices the rest" (Stick Together). Each keeps a
    /// largest party of their strongest creatures.
    EachPlayerKeepsPartySacrificesRest,
    /// Harper Recruiter: look at the top `count`, take a Cleric, Rogue, Warrior
    /// and/or Wizard (the largest party), bottom the rest at random.
    LookTopTakeParty { who: PlayerRef, count: Value },
    /// Hurkyl, Master Wizard: reveal the top `count`; take one card per card type
    /// among noncreature spells you cast this turn; the rest bottom at random
    /// (`game/effects/reveal_cast_types.rs`).
    RevealTopTakeOnePerCastType { who: PlayerRef, count: Value },
    /// Sculpted Sunburst: you choose a creature, each opponent one with equal or
    /// lesser power; exile every unchosen creature
    /// (`game/effects/sculpted_sunburst.rs`).
    ChooseCreaturesByPowerExileRest,
    /// "Return target creature to its owner's hand. Gain control of all Auras
    /// and Equipment that were attached to it, then attach them to another
    /// creature" (Fumble; `game/effects/fumble.rs`).
    BounceStealAttachmentsReattach { what: Selector },
    /// Threats Undetected: search up to `max` creatures with different powers; an
    /// opponent shuffles `back` of them in, the rest to hand
    /// (`game/effects/threats_undetected.rs`).
    SearchDistinctPowersOpponentReturnsTwo { max: u32, back: u32 },
    /// Musician's "{T}: Put a music counter on target creature. If it doesn't
    /// have [the upkeep toll], it gains that ability" (`game/effects/music.rs`).
    AddMusicCounterGrantUpkeepToll { what: Selector },
    /// Balduvian Warlord: remove a blocker from combat (its attackers unblocked
    /// unless blocked by another), then it blocks an attacker of your choice
    /// (`game/effects/reblock.rs`).
    RemoveBlockerThenReblock { what: Selector },
    /// Beamsplitter Mage — `CopyForEachOtherTargetableCreature` with one
    /// copy, aimed at the other creature its caster chooses
    /// (`game/effects/spell_copy_others.rs`).
    CopyForChosenOtherTargetableCreature,
    /// The Dragon-Kami Reborn: look at the top `count`, exile one face down with
    /// a `kind` counter, bottom the rest (`game/effects/hatching.rs`).
    LookTopExileOneFaceDownWithCounter { count: Value, kind: crate::card::CounterType },
    /// The rest of a token copy's "except" clause, stamped as copiable values on
    /// `Selector::LastCreatedTokens`: artifact subtypes and activated abilities
    /// `CreateTokenCopyOf` has no field for (Brenard's Food Golem).
    StampTokenCopyExceptions {
        artifact_subtypes: Vec<crate::card::ArtifactSubtype>,
        activated: Vec<ActivatedAbility>,
        /// Triggered abilities the copy "has" (Chandra, Flameshaper's "At the
        /// beginning of the end step, sacrifice this token") — copiable, so a
        /// copy of the token keeps them (CR 707.9b).
        #[serde(default)]
        triggered: Vec<TriggeredAbility>,
    },
    /// "Return this card attached to [host]" with a declared selector host, so an
    /// activated ability announces it (Gryff's Boon). `ReturnSelfAttachedToTarget`
    /// reads an undeclared slot 0.
    ReturnSelfAttachedTo { host: Selector },
    /// "[what] can't attack you or planeswalkers you control" for `duration`
    /// — `Keyword::CantAttackPlayer` naming the resolving controller.
    GrantCantAttackYou { what: Selector, duration: Duration },
    /// Orzhov Advokist: each player may put `counters` +1/+1 counters on their
    /// strongest creature; those who do can't attack you or your planeswalkers
    /// until your next turn.
    EachPlayerMayCounterForPeace { counters: u32 },
    /// Agitator Ant — "each player may put `counters` +1/+1 counters on a
    /// creature they control. Goad each creature that had counters put on it
    /// this way." The creature is the taker's greatest-power one.
    EachPlayerMayCounterThenGoad { counters: u32 },
    /// CR 701.38 secret council (Mob Verdict) — each player votes for another
    /// player; per vote, `on_opponent` runs with the voted-for opponent bound
    /// as `PlayerRef::Triggerer`, or `on_you` when the controller got it.
    EachPlayerVotesForAPlayer { on_opponent: Box<Effect>, on_you: Box<Effect> },
    /// Battlebond friend or foe (Zndrsplt's Judgment): the controller labels every
    /// living player, itself included; each then runs `friend` or `foe` as that
    /// player (`EachPlayerDoes`). Untargeted.
    FriendOrFoe { friend: Box<Effect>, foe: Box<Effect> },
    /// Gluntch, the Bestower: name a different living player per body; each body
    /// runs as its player (`EachPlayerDoes`). Untargeted (CR 115.10).
    ChooseDistinctPlayers(Vec<Effect>),
    /// CR 701.38 secret council for players, counts mattering (Círdan the
    /// Shipwright): `per_vote` per vote received, `unvoted` per player with none,
    /// each with that player as `PlayerRef::Triggerer`.
    SecretCouncilPlayerVote { per_vote: Box<Effect>, unvoted: Box<Effect> },
    /// CR 701.38 secret council for a permanent (Trap the Trespassers): votes for
    /// `filter` permanents (controller as "you"); `per_vote` per vote with the
    /// permanent as `Target(0)`.
    SecretCouncilPermanentVote { filter: SelectionRequirement, per_vote: Box<Effect> },
    /// CR 701.38 — each player secretly votes for up to one [filter]; no votes
    /// runs `on_none`, otherwise `on_most` once per top (tied) permanent as
    /// `Target(0)`. Abstaining is allowed. Vault 11: Voter's Dilemma.
    SecretCouncilPermanentVoteMost {
        filter: SelectionRequirement,
        on_most: Box<Effect>,
        on_none: Box<Effect>,
    },
    /// Prisoner's Dilemma — each opponent secretly chooses silence or snitch.
    /// All silence: `all_silence` damage to each. All snitch: `all_snitch` to
    /// each. Otherwise `mixed` to each opponent who chose silence.
    OpponentsChooseSilenceOrSnitch { all_silence: u32, all_snitch: u32, mixed: u32 },
    /// "Each opponent chooses [option]. For each player who chose X, …" (Master
    /// of Ceremonies): picks in turn order, then each pick's body runs with
    /// `PlayerRef::CurrentVoter` bound. Not a vote (CR 701.38): no ballot
    /// modifiers or `Voted` events.
    EachOpponentChooses { prompt: String, options: Vec<VoteOption> },
    /// Kwain, Itinerant Meddler — "Each player may draw a card, then each
    /// player who drew a card this way gains `life` life." Asked APNAP; the
    /// takers draw, then gain.
    EachPlayerMayDrawThenTakersGainLife { life: u32 },
    /// Turtles in Time — "Each player may shuffle their hand and graveyard
    /// into their library, then each player who does draws `draw` cards."
    /// Asked APNAP; every taker shuffles, then every taker draws.
    EachPlayerMayShuffleHandAndGraveyardThenDraw { draw: u32 },
    /// Mystic Barrier — "choose left or right", stamped on the source (left:
    /// toward the next seat, the CR 803.1a direction; right: the previous).
    ChooseAttackDirection,
    /// Teyo, Geometric Tactician's −2: Mystic Barrier's attack-direction
    /// restriction without a permanent, stamped on
    /// `ColdState::temporary_attack_direction` until the controller's next turn.
    ChooseAttackDirectionUntilYourNextTurn,
    /// Selective Obliteration: each player picks a color (their most common);
    /// exile every permanent that isn't colorless or only that color.
    EachPlayerChoosesColorExileOthers,
    /// Cloudform (CR 701.34): manifest the top card and attach this to it as an
    /// Aura with enchant creature (via `temporary_copies`, reverting as it
    /// leaves).
    ManifestTopAttachSource,
    /// Jeskai Infiltrator — "exile it and the top card of your library in a
    /// face-down pile, shuffle that pile, then manifest those cards."
    ExileSourceAndTopThenManifest,
    /// "As this enters, choose two players" (CR 614.12): the controller's
    /// pick from a ballot of pairs, stored in `ColdState::chosen_player_pairs`
    /// (Sower of Discord, Bitter Feud).
    ChooseTwoPlayersForSource {
        /// The headless pick is you and the most hostile opponent (Bitter
        /// Feud), not the two least-life opponents (Sower of Discord).
        #[serde(default)]
        with_you: bool,
    },
    /// Sower of Discord's trigger body — the damaged player (the trigger's
    /// `TriggerEventPlayer`) is one of the source's chosen pair: the other
    /// loses `amount` life. A no-op for anyone else.
    OtherChosenPlayerLosesLife { amount: Value },
    /// Aminatou, the Fateshifter's −6 — "choose left or right. Each player
    /// gains control of all nonland permanents other than [this] controlled
    /// by the next player in the chosen direction."
    RotateNonlandPermanents,
    /// Aminatou's Augury: this turn cast one spell of each nonland card type from
    /// `what` free; each cast spends one of its types
    /// (`Player::free_type_cast_budgets`).
    GrantFreeCastOnePerCardType { what: Selector },
    /// Yennett: reveal the top card; cast it free if it matches `filter`, else
    /// draw. `land_or_hand` (Ziatora's Envoy): a land may be played instead, and a
    /// miss goes to hand rather than drawing.
    CastTopFreeIfElseDraw {
        filter: crate::card::SelectionRequirement,
        #[serde(default)]
        land_or_hand: bool,
    },
    /// Djinn of Wishes — "reveal the top card of your library. You may play
    /// that card without paying its mana cost. If you don't, exile it." A
    /// land needs a land play left on your own turn.
    PlayTopFreeElseExile,
    /// "You may play lands from among [what]" during the resolution (Gix,
    /// Yawgmoth Praetor ruling): one land, on your turn with a land play left
    /// (CR 305.2b / 305.3). Pair with `CastAnyOrderWithoutPaying`.
    PlayLandAmongNow { what: Selector },
    /// Ulalek, Fused Atrocity — "copy all spells you control, then copy all
    /// other activated and triggered abilities you control" (CR 707.10).
    CopyAllSpellsAndAbilitiesYouControl,
    /// Benthic Anomaly: choose a creature per opponent; token-copy one of them
    /// with P/T equal to the choices' totals, as a colorless Eldrazi. Headless:
    /// each opponent's strongest, copying the priciest.
    CopyOnePerOpponentWithTotalStats,
    /// "Copy the next loyalty ability you activate this turn" `copies` times
    /// (Jaya's Phoenix) via a one-shot `LoyaltyCopyGrant`. `linked`: shares one
    /// "when you next" with the source's spell rider; whichever fires spends the
    /// other (`DropLinkedLoyaltyCopies`).
    CopyNextLoyaltyAbility {
        copies: u32,
        #[serde(default)]
        linked: bool,
    },
    /// Spend the source's pending `linked` loyalty-copy grant: its shared
    /// "when you next …" fired on a spell instead (Repeated Reverberation).
    DropLinkedLoyaltyCopies,
    /// "When you next … activate an ability with {X} in its activation cost
    /// this turn, copy that ability" (Magus Lucea Kane): a one-shot grant
    /// beside the spell rider; `linked` ties the two so either spends both.
    CopyNextXAbilityThisTurn { linked: bool },
    /// "When you next activate an ability that isn't a mana ability this turn
    /// by spending `min_mana` or more mana to activate it, copy that ability"
    /// (Dynaheir, Invoker Adept).
    CopyNextCostlyAbilityThisTurn { min_mana: u32 },
    /// Leori, Sparktouched Hunter: this turn, copy each activated ability of a
    /// planeswalker of the chosen type (your most common on battlefield and in
    /// hand).
    CopyLoyaltyAbilitiesOfChosenTypeThisTurn,
    /// Guff Rewrites History: owners shuffle the permanents in; each player who
    /// controlled one exiles until a nonland card, bottoms the rest, and may cast
    /// it free.
    ShuffleInThenCastFromTopFree { what: Selector },
    /// CR 901.9 — `who` rolls the planar die as an effect (no cost, no timing):
    /// blank, chaos, or planeswalk. Outside Planechase only chaos triggers and
    /// the roll matter.
    RollPlanarDie { who: PlayerRef },
    /// CR 701.31 — `who` planeswalks (Path of the Schemer's will of the
    /// planeswalkers).
    Planeswalk { who: PlayerRef },
    /// CR 309.6 / 901.9b — chaos ensues for `who` (Path of the Schemer).
    ChaosEnsues { who: PlayerRef },
    /// CR 701.60 — *suspect* each creature `what` resolves to: set its
    /// `suspected` flag so it gains menace and can't block (injected as
    /// computed keywords). Repeat Offender, Reasonable Doubt.
    Suspect { what: Selector },
    /// The inverse of [`Suspect`] — clear the `suspected` flag on every creature
    /// `what` resolves to ("~ are no longer suspected", Absolving Lammasu).
    ClearSuspected { what: Selector },
    /// CR 701.35 — detain: `detained_by` = the controller; it can't attack, block
    /// or activate until the detainer's next turn. Lyev Skyknight.
    Detain { what: Selector },
    /// CR 509.4 — create `definition` blocking the attacker in slot 0 (Flash
    /// Foliage; `filter` should require an attacker). No-op if it isn't
    /// attacking.
    CreateTokenBlocking { definition: std::sync::Arc<crate::card::TokenDefinition>, filter: SelectionRequirement },
    /// CR 509.4 — "create … that's blocking that creature" (Brimaz): the
    /// attacker the source blocks, not a target.
    CreateTokenBlockingThat { definition: std::sync::Arc<crate::card::TokenDefinition> },
    /// Mirror Match (CR 509.4 / 707.2): a blocking token copy of each creature
    /// attacking you or your planeswalkers, exiled at end of combat.
    CopyAttackersAsBlockers,
    /// Dawnbreak Reclaimer: you pick a creature card in an opponent's graveyard,
    /// they pick one in yours, and you may return both. An opponent with a
    /// creature card must be picked when one exists.
    ChooseGraveyardCreaturesEachMayReturn,
    /// Sphinx Ambassador: search `who`'s library for a card; they name a card;
    /// a creature not of that name may enter under your control; they shuffle.
    SearchTheirLibraryAgainstNamedCard { who: PlayerRef },
    /// Master of the Wild Hunt: tap your untapped [filter]s, each dealing its
    /// power to `target`, which divides its power among them. The division is
    /// asked first, of the target's controller; headless kills the most,
    /// smallest first.
    TapMatchingDealPowerAndTakeDivided { filter: SelectionRequirement, target: Selector },
    /// "For each player, you choose a creature that player controls. Then each
    /// player sacrifices all other creatures they control that don't share a
    /// creature type with the chosen creature they control" (Winnowing).
    EachPlayerSacrificesUnlikeChosen,
    /// Winter, Cynical Opportunist: exile any graveyard cards with `min_types`+
    /// card types among them; if you do, one permanent card among them enters
    /// with a finality counter. The engine picks (`effects/graveyard_spread.rs`).
    ExileTypeSpreadReturnPermanent { min_types: u32 },
    /// Sinister Waltz: of the spell's graveyard targets still there, `count` at
    /// random return, the rest bottomed (after an `ApplyToTargets` declaring
    /// the slots).
    ReturnTargetCardsAtRandom { count: Value },
    /// Scrap Mastery: each player exiles graveyard artifacts, sacrifices their
    /// artifacts, then returns the exiled ones; each step APNAP before the next.
    EachPlayerRecyclesArtifacts,
    /// "You and `who` each secretly choose a [filter] that player controls; that
    /// player sacrifices them" (Malik, Grim Manipulator). A permanent chosen
    /// twice is sacrificed once. Headless: you name their priciest, they their
    /// cheapest.
    YouAndPlayerChooseTheirPermanentsToSacrifice { who: PlayerRef, filter: SelectionRequirement },
    /// "[Target] of an opponent's choice" (Volcanic Offering): you name an
    /// opponent (headless: the most hostile), who picks a `filter` permanent read
    /// with *you* as controller; `body` runs with it as `Target(0)`. `chooser`
    /// names the picker instead (Bladegriff Prototype); `theirs` restricts to the
    /// chooser's own (Magus of the Arena). The pick must be legal for you to
    /// target (CR 115.4) unless `untargeted` (Wei Assassins). The spell's own
    /// targets follow from `Target(1)`. Nothing happens without a match.
    OpponentChoosesPermanentThen {
        filter: SelectionRequirement,
        body: Box<Effect>,
        #[serde(default)]
        chooser: Option<PlayerRef>,
        #[serde(default)]
        theirs: bool,
        #[serde(default)]
        untargeted: bool,
    },
    /// CR 309.5b / 309.6 — the finished dungeon leaves the game. Appended to the
    /// bottommost room's ability so removal happens as it finishes resolving.
    CompleteDungeon,
    /// CR 701.49 — venture into the dungeon: enter the first room of a chosen
    /// dungeon (auto: Lost Mine of Phandelver) or advance; room abilities resolve
    /// inline (`base::dungeons`). The final room completes it
    /// (`Player.dungeons_completed`, `EventKind::DungeonCompleted`).
    Venture,
    /// CR 727.2 — "venture into `dungeon`": like [`Effect::Venture`] but the
    /// dungeon is fixed rather than chosen (the initiative's Undercity). A
    /// player already in a different dungeon is unaffected.
    VentureInto { dungeon: String },
    /// CR 727 — `who` takes the initiative.
    TakeInitiative { who: PlayerRef },
    /// CR 701.29 — fateseal N: look at the top `amount` of each targeted
    /// opponent's library and bottom any number. AutoDecider bottoms none.
    Fateseal { who: PlayerRef, amount: Value },
    /// Look at the top `count`; any number to hand, the rest to the graveyard,
    /// losing `life_per_card` per card kept (Painful Truths family).
    /// AutoDecider keeps none; a scripted / `wants_ui` decider picks.
    DigToHandLoseLife { count: Value, life_per_card: Value },
    /// CR 701.59 — collect evidence N: the controller may exile graveyard cards
    /// totalling mana value `amount`+, and only then `then` runs. Sample
    /// Collector. The engine picks the cheapest qualifying set; `then`'s targets
    /// are chosen when the ability is put on the stack.
    CollectEvidence { amount: Value, then: Box<Effect> },
    /// CR 701.59 — collect evidence X: exile any graveyard cards, X their total
    /// mana value; then `then` with `ctx.x_value = X` (Incinerator of the Guilty).
    /// UI picks the set; the bot exiles everything when it opts in.
    CollectEvidenceX { then: Box<Effect> },
    /// CR 701.61 — forage (exile three graveyard cards, else sacrifice a Food) as
    /// an optional cost; if paid, `then` resolves.
    Forage { then: Box<Effect> },
    /// CR 701.57 — discover N: exile from the top until a nonland card with mana
    /// value ≤ `n`, cast it free or put it in hand; the rest bottom at random.
    /// Geological Appraiser.
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
    /// [`Effect::ExileTopUntilNonland`] with every card stamped
    /// `exiled_with = source` (Krang & Shredder's "a card exiled with").
    ExileTopUntilNonlandWithSource { who: PlayerRef },
    /// Exile from the top until a `filter` card, recording every card for
    /// `Selector::ExiledThisResolution` (Unpredictable Cyclone).
    ExileTopUntilMatching { filter: crate::card::SelectionRequirement },
    /// Demonlord Belzenlok: exile from the top until a nonland card, to hand;
    /// repeat while its mana value is `again_at_least`+, taking `damage_each`
    /// per card. An empty library stops it.
    ExileUntilNonlandToHandRepeat { again_at_least: u32, damage_each: u32 },
    /// Rally the Horde: exile the top `batch`, repeating while the last isn't a
    /// land; then `then` reads [`Value::NonlandCardsExiledThisEffect`].
    ExileTopBatchesUntilLandLast { batch: u32, then: Box<Effect> },
    /// "Reveal the top `count` cards of your library. An opponent chooses one
    /// of them. Put that card into your graveyard and the rest into your
    /// hand." Murmurs from Beyond — the pick is routed to an opponent's seat.
    RevealTopOpponentBinsOne {
        count: u32,
        /// Thran Tome — the unchosen cards stay on top of the library (to be
        /// drawn by a following clause) instead of going to hand.
        #[serde(default)]
        rest_stay_on_top: bool,
    },
    /// "You may cast a spell matching `filter` from among cards exiled with
    /// this source without paying its mana cost." Kaho, Minamo Historian —
    /// `filter`'s X resolves against the activation's chosen X.
    MayCastExiledWithSource { filter: crate::card::SelectionRequirement },
    /// Exile from the top of `who`'s library until a nonland card, which you may
    /// play for `duration`; the others stay exiled (Territorial Bruntar). Unlike
    /// `Discover`: no mana value cap and no bottoming.
    ExileTopUntilNonlandMayPlay {
        who: PlayerRef,
        duration: crate::card::MayPlayDuration,
        /// `true` grants a free cast; otherwise the caster pays the card's
        /// normal mana cost from exile.
        #[serde(default)]
        free: bool,
        /// `Some`: the may-play applies only below this mana value, else the card
        /// goes to hand (Solstice Revelations).
        #[serde(default)]
        hand_unless_mv_below: Option<Value>,
        /// Grant the cast to the exiling player (library owner), not the controller
        /// (Transforming Flourish). `false` (default) is the impulse-draw family.
        #[serde(default)]
        grant_to_exiling_player: bool,
    },
    /// Ignis Scientia: look at the top `count`, a land may enter tapped, bottom
    /// the rest at random (Impulse machinery, `to_battlefield` + `tapped`).
    DigForLandToBattlefield { count: Value },
    /// CR 702.39 — *provoke*: untap the creature `what` resolves to and force
    /// it to block this combat's source attacker if able. Sets the target's
    /// `must_block` to the effect source. Used by `shortcut::provoke`.
    Provoke { what: Selector },
    /// CR 701.44 — each `who` permanent explores: a revealed land goes to hand,
    /// otherwise a +1/+1 counter and the card stays on top (the graveyard option
    /// is collapsed). An empty library still explores and grants the counter.
    /// Emits `GameEvent::Explored`.
    Explore { who: Selector },
    /// One explore with no replacement left — what `Explore` becomes after Twists
    /// and Turns' scry (`game/explore.rs`). Engine-built only.
    ExploreAfterScry { who: Selector },
    /// Look at the top `count`, one to hand, the rest on the bottom (or the
    /// graveyard with `rest_to_graveyard`) — Impulse. Picked via the
    /// `SearchLibrary` decision (auto keeps the top card). `pick_filter`
    /// restricts what may be taken (Satyr Wayfinder's land); `None` = anything.
    LookPickToHand(Box<LookPick>),
    /// Look at the top `count`; keep one on top, mill the rest (Sage of Days),
    /// via the `ImpulsePending` machinery with `keep_on_top: true`. `who:
    /// Some(..)` reads another player's library and auto-picks (lowest mana
    /// value kept on an opponent's — Dimir Charm). `exile_rest` exiles instead
    /// (Devourer of Destiny); `rest_bottom_random` bottoms them (Aladdin's
    /// Lamp).
    LookTopKeepOneRestToGraveyard {
        count: Value,
        #[serde(default)]
        who: Option<PlayerRef>,
        #[serde(default)]
        exile_rest: bool,
        #[serde(default)]
        rest_bottom_random: bool,
    },
    /// CR 702.179f — if `who`'s speed is greater than each other player's,
    /// reduce it by 1, not below 1 (Spikeshell Harrier).
    ReduceSpeedIfFastest { who: PlayerRef },
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
    /// Tempting offer (CR 207.2c): `body` runs for the controller, each opponent
    /// may run it too, and the controller re-runs it per acceptor (Tempt with
    /// Bunnies). `declined` runs under each opponent who didn't (Wernog).
    TemptingOffer {
        body: Box<Effect>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        declined: Option<Box<Effect>>,
    },

    /// CR 101.4 — "for each opponent, [body]": `body` once per opponent in APNAP
    /// order, bound as `PlayerRef::Triggerer` / `Selector::TriggerSource`, for
    /// bodies that must name *that* opponent (Adeline), which `EachOpponent`
    /// can't. One run in a duel.
    ForEachOpponent { body: Box<Effect> },

    /// CR 207.2c join forces: every seat, from the controller in turn order, may
    /// pay any mana; `body` runs once with the total as
    /// `Value::TriggerEventAmount` (Minds Aglow). All asks precede payment, so a
    /// suspend-and-replay is idempotent.
    JoinForces { description: String, body: Box<Effect> },
    /// "[who] may [accept]. If a player does, [`if_any`]; if no one does,
    /// [`otherwise`]." Every player in `who` is asked in APNAP order regardless
    /// of earlier answers (Argothian Wurm and Desecration Demon rulings). Each
    /// accepter runs `on_accept` bound to slot 0; `if_any` runs once however many
    /// accepted. CR 101.4a puts every choice before any action, which also makes
    /// a mid-ask suspend replay-safe. Vexing Devil, Browbeat.
    PlayersMayAccept {
        who: PlayerRef,
        description: String,
        on_accept: Box<Effect>,
        if_any: Box<Effect>,
        otherwise: Box<Effect>,
    },
    /// "If a creature would enter the battlefield under an opponent's
    /// control this turn, it enters under your control instead." Gather
    /// Specimens (CR 614 control-ETB replacement; expires at cleanup).
    StealCreatureEtbThisTurn,
    /// Crafty Cutpurse (CR 614): opponents' tokens this turn are created under
    /// you (`steals_opponent_tokens_this_turn`, until the next untap).
    StealOpponentTokensThisTurn,
    /// Oversimplify: exile all [filter]; each player makes `definition` with
    /// counters equal to their exiled ones' total power (read before; a token
    /// counts, CR 111.7).
    ExileAllThenTokenPerPlayerByPower {
        filter: SelectionRequirement,
        definition: std::sync::Arc<crate::card::TokenDefinition>,
    },
    /// Dakkon, Shadow Slayer −6: put a `filter` card from hand or graveyard onto
    /// the battlefield (auto: priciest; UI via the search decision).
    PutFromHandOrGraveyardOntoBattlefield { filter: crate::card::SelectionRequirement },
    /// Reveal the top card to hand; `who` loses its mana value (Sorin, Grim
    /// Nemesis +1). `you_gain` also gains the controller that much (Twilight
    /// Prophet).
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
        /// "Otherwise, put it into your graveyard" (Call of the Wild) instead
        /// of leaving the miss on top.
        #[serde(default)]
        miss_to_graveyard: bool,
        /// The hit enters tapped (Galadriel of Lothlórien's land).
        #[serde(default)]
        tapped: bool,
    },
    /// Moonlight Bargain: look at the top `count`; each goes to the graveyard
    /// unless you pay `life`, the rest to hand. Auto buys while above 1 life.
    LookTopEachPayLifeOrBin { count: Value, life: u32 },
    /// Sylvan Library: choose `count` hand cards drawn this turn; per card pay
    /// `life` or put it on top (in chosen order). "Drawn this turn" is the last
    /// `cards_drawn_this_turn` cards of the hand, so a card added after a draw
    /// can stand in.
    PayLifeOrPutBackDrawnThisTurn { count: u32, life: u32 },
    /// CR 708.2a — turn each resolved permanent face down: it becomes a 2/2
    /// creature with no name, types, or abilities. A no-op on a permanent
    /// that is already face down (CR 708.2b). Ixidron.
    TurnFaceDown { what: Selector },
    /// [`Effect::TurnFaceDown`] with one listed creature type: "turn them face
    /// down. They're 2/2 Horror creatures" (Illithid Harvester, CR 708.2).
    TurnFaceDownAs { what: Selector, creature_type: crate::card::CreatureType },
    /// CR 614 — "the next time you would draw this turn, [`body`] instead"
    /// (Onslaught Words): stacked one-shot charges, spent front first, gone at
    /// end of turn.
    ReplaceYourNextDrawThisTurn { body: Box<Effect> },
    /// CR 707.9 — turn each face-down pick face up without paying its morph cost
    /// (Break Open). One whose face-up side isn't a creature card (a cloaked
    /// land) can't be; `if_cant` runs with it as target 0 (Etrata, Deadly
    /// Fugitive).
    TurnFaceUpFree { what: Selector, if_cant: Option<Box<Effect>> },
    /// Concerted Effort: each of `keywords` one of your creatures has (computed,
    /// read once) goes to all your creatures until end of turn.
    ShareKeywordsAmongYourCreatures { keywords: Vec<Keyword> },
    /// Tunnel Vision: name a card; target player reveals until it; if found the
    /// rest are binned and it goes back on top, else they shuffle.
    NameCardRevealUntilThenBin { who: PlayerRef },
    /// "`who` chooses a card name, then reveals the top card of their
    /// library. On a match it goes to their hand, otherwise to their
    /// graveyard." Petra Sphinx.
    NameCardThenRevealTopBin { who: PlayerRef },
    /// Exile every permanent card sacrificed to pay the *current* activation's
    /// cost (Sword of the Ages' "then exile this artifact and those creature
    /// cards"). Reads the batch stashed by `activate_ability`.
    ExileCostSacrificedBatch,
    /// Divining Witch: exile the top `exile_count`, then reveal until the card
    /// the source named (`Effect::NameCard`), take it, exile the rest.
    ExileTopThenRevealUntilNamed { exile_count: Value },
    /// "Reveal the top `count` cards of your library and put all of them with
    /// the name a preceding `Effect::NameCard` stamped into your hand. Exile
    /// the rest." Desperate Research.
    RevealTopTakeNamedExileRest { count: Value },
    /// Global Ruin: each player keeps one land of each basic type (first, or a
    /// `ChooseCards` pick for UI) and sacrifices the rest.
    EachPlayerKeepsOneOfEachBasicTypeSacrificesRest,
    /// Stronghold Gambit: each player reveals a chosen hand card
    /// (`Decision::ChooseCards`; auto: cheapest creature, else cheapest card);
    /// owners of the lowest-mana-value creature cards put them onto the
    /// battlefield.
    RevealChosenCardsLowestCreaturesEnter,
    /// "Until end of turn, [what] loses 'Prevent all damage that would be
    /// dealt to this'" (Glittering Lion / Lynx). Sets
    /// `CardInstance.damage_prevention_off_eot`.
    TurnOffDamagePreventionThisTurn { what: Selector },
    /// CR 611.2 — "this turn, whenever a [`filter`] …": a floating watcher every
    /// matching permanent carries for the turn, later arrivals included (Mage
    /// Hunters' Onslaught). The matching permanent is the trigger's source.
    GrantTriggeredAbilityThisTurnToMatching {
        filter: crate::card::SelectionRequirement,
        trigger: Box<crate::card::TriggeredAbility>,
    },
    /// CR 611.2 — "Whenever a creature attacks this turn, it gets +P/+T until
    /// end of turn" (Song of Blood). The bonus is fixed when this resolves, so
    /// a later trigger can't re-read a resolution-scoped tally as zero.
    PumpAttackersThisTurn { power: Value, toughness: Value },
    /// CR 509 — "Attacking creatures become blocked" (Fog Patch), even ones
    /// that can't be blocked. No blocker is assigned, so nothing deals or is
    /// dealt combat damage by the block.
    AttackingCreaturesBecomeBlocked,
    /// Portal Manipulator: at resolution pick a player (headless: whoever can
    /// take the most), then every attack on the controller or its planeswalkers
    /// by a creature the controller could target moves to that player. Not a new
    /// attack: restrictions, costs and attack triggers are ignored.
    AttackersNowAttackChosenPlayer,
    /// "You may reselect which player or permanent [what] is attacking"
    /// (Misleading Signpost): ignores restrictions and costs, and is not a new
    /// attack (no attack triggers). `players_only`: no planeswalker
    /// (Capricopian).
    ReselectAttackTarget {
        what: Selector,
        #[serde(default)]
        players_only: bool,
    },
    /// Look at the top `count` of `who`'s library (default an opponent), exile
    /// one (auto: priciest), bottom the rest; `grant` says what it may then do
    /// (Gonti, Vivien's −2).
    LookTopExileOneMayPlay {
        count: Value,
        #[serde(default = "player_ref_target_zero")]
        who: PlayerRef,
        #[serde(default)]
        grant: LookExileGrant,
    },
    /// Planar Genesis: look at the top `count`; a land may enter tapped, else
    /// take a card; the rest bottom at random. Prefers deploying a land, else
    /// takes the priciest card.
    LookTopDeployLandOrHand { count: Value },
    /// "Look at the top card of your library. If it's a land card, you may put
    /// it onto the battlefield (tapped if `tapped`)." A non-land — or a decline —
    /// stays on top (unlike the bottoming look-top variants). Mobile Homestead.
    LookTopMayDeployLand { tapped: bool },
    /// Look at the top card; if it matches `filter` you may reveal it and take
    /// it, else bottom it (Vivien's Grizzly). The yes/no is a
    /// `Decision::OptionalTrigger`, suspending like `Effect::MayDo`.
    LookTopMayRevealMatchToHandElseBottom { filter: SelectionRequirement },
    /// Cabal Therapy: choose a nonland card name; target player discards
    /// every card with that name from their hand.
    NameCardTargetDiscardsMatching,
    /// "Target player reveals their hand and discards all `filter` cards"
    /// (Trapfinder's Trick). The reveal is knowledge-only; every match is
    /// discarded through the normal discard path so discard triggers fire.
    RevealHandDiscardAllMatching { who: PlayerRef, filter: SelectionRequirement },
    /// Sirocco: target player reveals their hand and discards each `filter` card
    /// unless they pay `life` (asked per card; unaffordable = decline).
    RevealHandDiscardMatchingUnlessPayLife {
        who: PlayerRef,
        filter: SelectionRequirement,
        life: u32,
    },
    /// "Target player reveals their hand" (Darigaaz, the Igniter). Knowledge
    /// only: the hand becomes visible to the resolving controller for the rest
    /// of the game via `hands_revealed_to`.
    RevealHand { who: PlayerRef },
    /// The player reveals their top card (Aven Windreader; every player for
    /// Haunting Imitation); visible to all while it stays on top.
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
    /// "Each resolved player loses all counters" — every player counter kind
    /// the engine tracks (poison, energy, experience, rad). Final Act's
    /// fifth mode.
    RemoveAllPlayerCounters { who: PlayerRef },
    /// "Target player loses all rad counters" (Survivor's Med Kit's RadAway)
    /// — rad only, unlike `RemoveAllPlayerCounters`.
    RemoveAllRadCounters { who: PlayerRef },
    /// "Remove `count` `kind` counters from among your [filter]s. When you do,
    /// [then]" — resolution-time sibling of
    /// `ActivatedAbility::remove_counter_among_filter` (Overseer of Vault 76).
    /// Nothing unless all `count` can go; drains weakest first. Wrap in `MayDo`
    /// for "you may".
    RemoveCountersFromAmongThen {
        kind: crate::card::CounterType,
        count: u32,
        filter: SelectionRequirement,
        then: Box<Effect>,
    },
    /// "Remove a counter from a [filter] you control. If you do, [then]" (Mister
    /// Hyde) — any kind. Headless removes what a seat least wants: stun, -1/-1,
    /// other kinds, then +1/+1 and loyalty; ties to the weakest permanent.
    RemoveAnyCounterFromAmongThen { filter: SelectionRequirement, then: Box<Effect> },
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
    /// Liar's Pendulum: name a card; an opponent guesses whether it's in your
    /// hand (`OptionalTrigger`); if wrong, reveal and draw.
    LiarsPendulum,
    /// Gollum, Scheming Guide: the controller picks land or nonland (game RNG —
    /// a bluff is policy), the first opponent guesses (`OptionalTrigger`), the
    /// top card is revealed (CR 701.20); `right` or `wrong` runs. An empty
    /// library is a right guess.
    GuessTopCardKind { right: Box<Effect>, wrong: Box<Effect> },
    /// Scythe of the Wretched: a creature its host damaged this turn died (per
    /// LKI `damaged_by_this_turn`); return it under you and attach the Equipment.
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
    /// Tamiyo, Collector of Tales +1: name a nonland card, reveal the top
    /// `count`, take every copy, bin the rest.
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
    /// "Create a copy of this card. You may cast the copy." — paying its costs
    /// (CR 707.12; Q, Stacking the Deck's granted ability). The copy is staged
    /// in exile as a token and ceases to exist if it isn't cast.
    CopyCardMayCast { what: Selector },
    /// Myra the Magnificent: exile `what` and link it to a chosen Attraction you
    /// control without a midway counter, which gets one plus "whenever you visit
    /// this, copy the exiled card; you may cast the copy free" (CR 718). No such
    /// Attraction: just the exile.
    ExileOntoAttractionWithMidway { what: Selector },
    /// "Copy those cards. You may cast up to `max` copies free" (Baron Helmut
    /// Zemo, over `Selector::CostExiledCards`): each card offered as a
    /// `CopyCardAndCastFree`, priciest first; uncast copies cease to exist (CR
    /// 707.12).
    CopyCardsAndCastFreeUpTo { what: Selector, max: u32 },
    /// Uldaros Theorix: copy those cards and cast copies free with total mana
    /// value ≤ `total`, priciest first while each fits.
    CopyCardsAndCastFreeTotalMv { what: Selector, total: Value },
    /// Summoner's Egg: turn the imprinted card face up; a creature enters under
    /// the source's controller, anything else stays exiled.
    RevealImprintDeployCreature,
    /// "Target opponent reveals their hand. You may copy an instant or sorcery
    /// card in it. If you do, you may cast the copy without paying its mana
    /// cost." Reversal of Fortune.
    ReversalOfFortune,
    /// "Each player sacrifices a permanent of their choice unless they discard
    /// a card" (Possessed Portal's end step). Each player is asked in APNAP
    /// order; the auto-picker discards when it can.
    EachPlayerSacrificesUnlessDiscards,
    /// Lonis, Genetics Expert: target opponent reveals the top `count`; you may
    /// put a nonland permanent with mana value ≤ `count` onto the battlefield
    /// under your control; they shuffle.
    RevealOpponentTopPutOntoBattlefield { count: Value, filter: SelectionRequirement },
    /// Winota, Joiner of Forces: look at the top `count`; a `filter` card may
    /// enter tapped and attacking the defender `trigger_source` attacks, with
    /// indestructible until end of turn; the rest bottom at random. Auto-picks
    /// the highest power; no-op outside combat. `return_at_end_of_combat`
    /// replaces indestructible with a return to hand (Arthur, Marigold Knight).
    LookTopMayDeployAttacking {
        count: Value,
        filter: SelectionRequirement,
        #[serde(default)]
        return_at_end_of_combat: bool,
        /// Neither rider: the creature just stays (Owlbear Cub's "Mama's
        /// Coming").
        #[serde(default)]
        plain: bool,
    },
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
    LookTopMayBottomAllElse {
        /// Whose library is looked at. `None` = the controller's (the Petals
        /// of Insight default); `Some` reads another seat's top — Coral
        /// Fighters looks at the *defending player's* top card.
        #[serde(default)]
        who: Option<PlayerRef>,
        count: Value,
        then: Box<Effect>,
        else_: Box<Effect>,
    },
    /// "For each [filter], return it to its owner's hand unless that permanent's
    /// controller pays `cost`." Cut the Tethers — one pay-or-bounce decision per
    /// permanent, asked of its controller.
    ReturnEachUnlessPays { filter: SelectionRequirement, cost: crate::mana::ManaCost },
    /// "For each [filter], its controller sacrifices a permanent of their
    /// choice unless they pay `cost`" (Fade Away). One pay-or-sacrifice
    /// decision per matching permanent, asked of its controller.
    SacrificeEachUnlessPays { filter: SelectionRequirement, cost: crate::mana::ManaCost },
    /// Phyrexian Grimoire: `who` picks one of your top two graveyard cards to
    /// exile; the other to hand (a lone card goes straight to hand).
    TopTwoGraveyardOpponentSplits { who: Selector },
    /// Interdict: the permanents' activated abilities can't be activated this
    /// turn (`GameState.abilities_locked_this_turn`; mana abilities exempt).
    LockActivatedAbilitiesThisTurn { what: Selector },
    /// Tatsumasa, the Dragon's Fang: create `definition`; return this card
    /// (exiled by `exile_self_cost`) when the token dies.
    CreateTokenReturnSelfWhenItDies { definition: std::sync::Arc<crate::card::TokenDefinition> },
    /// Firion — reduce the generic portion of each resolved permanent's
    /// printed Equip cost by `amount` (stamped onto the minted token copy).
    ReduceEquipCost { what: Selector, amount: u32 },
    /// Register a delayed trigger sacrificing each resolved permanent at the
    /// beginning of the controller's next upkeep (Firion's copy token).
    SacrificeAtNextUpkeep { what: Selector },
    /// "Exile those creatures at the beginning of your next upkeep" (Rally the
    /// Ancestors) — the exile twin of `SacrificeAtNextUpkeep`.
    ExileAtYourNextUpkeep { what: Selector },
    /// "Sacrifice them at the beginning of the next end step" (Pull). The
    /// selector is resolved now; each hit gets its own CR 603.7a delayed
    /// trigger.
    SacrificeAtNextEndStep { what: Selector },
    /// "Destroy it at the beginning of the next end step" (Old Hob, Alleycat
    /// Blues' Mutant) — the destroy twin of `SacrificeAtNextEndStep`, so an
    /// indestructible object survives it.
    DestroyAtNextEndStep { what: Selector },
    /// "Return that creature to its owner's hand at the beginning of the next
    /// end step" — a per-permanent delayed return (Zara, Renegade
    /// Recruiter), the hand-bound sibling of `SacrificeAtNextEndStep`.
    ReturnToOwnersHandAtNextEndStep { what: Selector },
    /// "Exile it at the beginning of the next end step" (Shallow Grave) — the
    /// exile twin of [`Effect::SacrificeAtNextEndStep`]. The selector is
    /// resolved now; each hit gets its own CR 603.7a delayed trigger.
    ExileAtNextEndStep { what: Selector },
    /// [`Effect::ExileAtNextEndStep`] with an intervening "unless": `unless`
    /// is read as each delayed trigger resolves (Tilonalli's Summoner's
    /// "exile those tokens unless you have the city's blessing").
    ExileAtNextEndStepUnless { what: Selector, unless: Predicate },
    /// Phyrexian Dreadnought: sacrifice this unless you sacrifice `filter`s with
    /// total power `total_power`+ (never the source). An auto seat pays only if
    /// it costs fewer bodies than the source is worth, so usually declines.
    SacrificeSourceUnlessSacrificeTotalPower { filter: SelectionRequirement, total_power: Value },
    /// Tainted Specter: the victim discards unless they put a hand card on top;
    /// `then` only on a discard. Empty hand: neither.
    DiscardUnlessPutCardOnTop { who: PlayerRef, then: Box<Effect> },
    /// "This creature assigns no combat damage this turn" (Kukemssa Pirates'
    /// price for the artifact). Flags each resolved permanent for the rest of
    /// the turn; combat damage assignment skips it.
    AssignsNoCombatDamageThisTurn { what: Selector },
    /// CR 603.7a — "At the beginning of your next upkeep, `body`." A general
    /// one-shot delayed trigger bound to the resolving source (Giant Slug,
    /// Hazezon Tamar).
    AtYourNextUpkeep { body: Box<Effect> },
    /// CR 603.7a — "At the beginning of the next turn's upkeep, `body`."
    /// Fires on the very next upkeep step, whoever's turn it is; the Homelands
    /// cantrip rider (Headstone, Jinx, Prophecy, Renewal).
    AtNextTurnsUpkeep { body: Box<Effect> },
    /// CR 702.141 Encore: per opponent, a hasty token copy of the source (read
    /// from exile) that must attack them, sacrificed at the next end step.
    EncoreTokens,
    /// Memories Returning: reveal five; you take one, an opponent bottoms one,
    /// twice each (you: priciest; they: priciest remaining).
    RevealFiveDraftAgainstOpponent,
    /// Choco: look at the top `count`; one to hand (priciest nonland), lands
    /// among the rest enter tapped, the remainder to the graveyard.
    LookTopTakeOneDeployLandsRestGraveyard { count: Value },
    /// Triple Triad — each player exiles their library top. Until end of
    /// turn the controller may play the card they own exiled this way for
    /// free, plus each other card exiled this way with lesser mana value.
    ExileEachTopFreePlayLesser,
    /// Random Encounter: shuffle, mill `amount`; milled creatures enter with
    /// haste and return to hand at the next end step.
    MillDeployCreaturesUntilEndStep { amount: Value },
    /// Gilgamesh: look at the top `count`; all `filter` cards enter, the rest
    /// bottom at random; if any entered, `then` with them on `LastMoved`.
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
        /// The remainder goes to the graveyard instead — "Then put all cards
        /// revealed this way that weren't put onto the battlefield into your
        /// graveyard" (Saheeli's Directive).
        #[serde(default)]
        rest_to_graveyard: bool,
        /// With `max`, the controller picks which matches (CR 608.2d) and may
        /// take none; `mandatory` makes it "put a [card]" (Cavalier of Thorns).
        #[serde(default)]
        mandatory: bool,
    },
    /// Atraxa, Grand Unifier: reveal the top `count`; take one card per card type
    /// present (a multi-type card once), bottom the rest at random.
    RevealTopTakeOnePerType { who: PlayerRef, count: Value },
    /// Reveal the top `count` of `who`'s library; every `filter` match to the
    /// graveyard, the rest bottom at random (The Fourteenth Doctor).
    RevealTopMatchingToGraveyard { who: PlayerRef, count: Value, filter: SelectionRequirement },
    /// Reveal the top `count` of `who`'s library; every `filter` match to hand,
    /// the rest bottom at random (CR 401.4). "Any number" is take-all (Torsten,
    /// Founder of Benalia).
    RevealTopTakeMatchingToHand {
        who: PlayerRef,
        count: Value,
        filter: SelectionRequirement,
        /// "…with different powers" (Rip, Spawn Hunter): at most one card of
        /// each printed power is taken, greediest-first.
        #[serde(default)]
        distinct_powers: bool,
    },
    /// Reveal the top `count` of `who`'s library: `filter` matches to hand, the
    /// rest to the graveyard (Borborygmos Enraged).
    RevealTopTakeMatchingRestToGraveyard {
        who: PlayerRef,
        count: Value,
        filter: SelectionRequirement,
    },
    /// Vigean Intuition: choose a card type, then reveal the top `count` cards
    /// of your library; put every card of the chosen type into your hand and
    /// the rest into your graveyard.
    ChooseTypeRevealTopPartition { count: Value },
    /// Portent of Calamity: reveal the top `count`; exile one per card type
    /// (each exiled *for* one type), the rest to the graveyard; with
    /// `free_cast_at`+ exiled you may cast one free; the other exiled cards go to
    /// hand.
    RevealTopExileOnePerCardType { count: Value, free_cast_at: u32 },
    /// Fertile Imagination: choose a card type; target opponent reveals their
    /// hand; create `per` 1/1 green Saproling tokens for each card of the
    /// chosen type revealed this way.
    FertileImagination { per: Value },
    /// Guild Feud: an opponent and then you each reveal three, may deploy a
    /// creature, bin the rest; two deployed creatures fight.
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
    /// Harness Infinity: swap `who`'s hand and graveyard as zone vectors (CR
    /// 701.10-style); per-card graveyard triggers don't fire.
    ExchangeHandAndGraveyard { who: PlayerRef },
    /// "Exchange your hand and library, then shuffle" (Mordenkainen's −10):
    /// the hand becomes the library (shuffled) and the library the hand.
    ExchangeHandAndLibrary { who: PlayerRef },
    /// Doomsday Excruciator: each `who` player exiles all but the bottom `keep`
    /// cards of their library (face up; face-down exile isn't modeled).
    ExileLibraryExceptBottom { who: PlayerRef, keep: Value },
    /// Mirror of Fate: choose up to `max` of your face-up exiled cards, exile
    /// your library, put the chosen on top. The choice comes first.
    ExileLibraryThenTopFromExile { max: u32 },

    // ── Zone moves ───────────────────────────────────────────────────────────
    /// Move every entity the selector resolves to into `to`.
    Move { what: Selector, to: ZoneDest },
    /// CR 401.4 — "put the rest on the bottom of your library in a random
    /// order": every card `what` resolves to goes to the bottom of its
    /// owner's library, the group shuffled first (Sanwell, Avenger Ace).
    BottomInRandomOrder { what: Selector },
    /// "Return any number of target [filter] cards with total mana value `cap` or
    /// less" (March from the Tomb): cheapest first until the next would bust the
    /// budget. No-op with no match; moved cards are `Selector::LastMoved`.
    MoveWithinTotalManaValue {
        from: Selector,
        filter: SelectionRequirement,
        cap: Value,
        to: ZoneDest,
        /// "Up to N" — at most this many cards (The War in Heaven's three).
        #[serde(default)]
        max_count: Option<u32>,
        /// The budget is total **power**, not mana value (Moorland Rescuer's
        /// "total power X or less").
        #[serde(default)]
        by_power: bool,
    },
    /// "[Return/put/exile] up to `count` cards of your choice from `from`" — the
    /// player-chooses sibling of `Move { what: Take {..} }`, via
    /// `Decision::ChooseCards` (auto takes the first `count`). `filter` narrows
    /// the set (Bind to Life: `from: LastMoved`, Creature). Pull from the Grave.
    MoveChosen {
        from: Selector,
        #[serde(default)]
        filter: Option<SelectionRequirement>,
        count: Value,
        #[serde(default)]
        up_to: bool,
        to: ZoneDest,
    },
    /// Quarry Colossus: put `what` beneath the top `count` cards of its owner's
    /// library (dynamic `FromTop`; CR 401.7 bottoms it if shorter).
    PutIntoLibraryBeneathTop { what: Selector, count: Value },
    /// Search `who`'s library for a card matching `filter` and move to `to`.
    Search { who: PlayerRef, filter: SelectionRequirement, to: ZoneDest },
    /// "Search for any number of `filter` cards, put them into `to`, shuffle"
    /// (Ugin, Eye of the Storms); moved cards feed `ExiledThisResolution` /
    /// `LastMoved`. CR 701.24c — shuffles even on an empty pick.
    SearchAnyNumber { who: PlayerRef, filter: SelectionRequirement, to: ZoneDest },
    /// Transmute Artifact: search for an artifact; it enters if its mana value is
    /// at most the sacrificed one's (`sacrificed_mana_value`), else pay the
    /// difference or it goes to the graveyard.
    TransmuteArtifact,
    /// "Search your graveyard, hand, and/or library" for a `filter` card to `to`
    /// (Dark Supplicant), pooling the listed zones. Listing [`Zone::Library`]
    /// makes it a real library search (shuffle, taxes / locks,
    /// `PlayerSearchedLibrary`).
    SearchZones {
        who: PlayerRef,
        zones: Vec<crate::card::Zone>,
        filter: SelectionRequirement,
        to: ZoneDest,
    },
    /// Gaea's Balance: search one land per basic type and put them onto the
    /// battlefield, then shuffle; unmatched types are skipped.
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
    /// "Search for up to `count` `filter` cards into `to`" as a chain of single
    /// `Search` picks (each reusing the `SearchPending` suspend). AutoDecider
    /// finds none. Nylea's Intervention. "Different names" is
    /// `R::NameDiffersFromLastMoved` in the filter.
    SearchUpToN { who: PlayerRef, filter: SelectionRequirement, to: ZoneDest, count: Value },
    /// CR 701.23 — `body`'s searches count as one: one tax and one search event
    /// (Cultivate's two picks; `game/search_batch.rs`).
    OneSearch { body: Box<Effect> },
    /// CR 701.23a — `picker` searches `who`'s library: the pick decision
    /// routes to `picker`'s seat, not the library's owner (Hide // Seek's
    /// "search target opponent's library ... exile that card").
    SearchPickedBy { who: PlayerRef, picker: PlayerRef, filter: SelectionRequirement, to: ZoneDest },
    /// CR 701.23 — "Search your library for any number of [`filter`] cards,
    /// exile them, then create that many tokens. Then shuffle." (Myr
    /// Incubator.) The auto-picker takes every match.
    SearchExileThenTokensPerCard {
        filter: SelectionRequirement,
        definition: std::sync::Arc<crate::card::TokenDefinition>,
    },
    /// CR 701.52 — seek `count` `filter` cards: random picks from `who`'s
    /// library, no choice and no shuffle (Tarkir: Dragonstorm).
    Seek { who: PlayerRef, filter: SelectionRequirement, count: Value, to: ZoneDest },
    /// Return `count` random `filter` cards from `who`'s graveyard to hand
    /// (Charmbreaker Devils); stops when out of matches.
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
    /// Elixir: shuffle every `filter` card from `who`'s graveyard into their
    /// library; they gain life equal to the count.
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
    /// Gideon's Sacrifice: this turn, damage to you and your permanents goes to
    /// the chosen permanent (CR 614.9; `damage_redirect_this_turn`, read by
    /// `damage_redirect_target`). `creatures_only`: only you and your creatures
    /// (Heroic Sacrifice).
    RedirectYourDamageToChosen {
        what: Selector,
        #[serde(default)]
        creatures_only: bool,
    },
    /// Turn the Tables: this turn, combat damage to you is dealt to `what`
    /// instead (`combat_damage_redirect_this_turn`).
    RedirectYourCombatDamageToTarget { what: Selector },
    /// Shriveling Rot mode 1 — "until end of turn, whenever a creature is dealt
    /// damage, destroy it". The lethal-damage SBA treats *any* marked damage as
    /// lethal for the rest of the turn (indestructible still survives).
    DamagedCreaturesDieThisTurn,
    /// Poison Breath: `DamagedCreaturesDieThisTurn` limited to your opponents'
    /// creatures.
    DamagedOpponentCreaturesDieThisTurn,
    /// Seasons Past: return any graveyard cards with different mana values to
    /// hand (at most one per value); `max` caps the count (Long Rest). Returns
    /// are on `Selector::LastMoved`.
    ReturnDistinctManaValuesFromGraveyard {
        who: PlayerRef,
        #[serde(default)]
        max: Option<Value>,
    },
    /// Shriveling Rot mode 2 — "until end of turn, whenever a creature dies,
    /// that creature's controller loses life equal to its toughness".
    CreatureDeathsDrainToughnessThisTurn,
    /// Deliver Unto Evil: up to `max_targets` graveyard targets (slots
    /// `0..max_targets`). With a Bolas planeswalker all return to hand;
    /// otherwise an opponent leaves two behind.
    DeliverUntoEvil { max_targets: u8, filter: SelectionRequirement },
    /// Nicol Bolas, Dragon-God's +1 rider — "Each opponent exiles a card from
    /// their hand or a permanent they control." Each opponent chooses one
    /// object among their hand cards and permanents to exile.
    EachOpponentExilesHandCardOrPermanent,
    /// CR 701.51 — "Open an Attraction": move the top card of your Attraction
    /// deck onto the battlefield under your control, face up. No-op when the
    /// Attraction deck is empty.
    OpenAnAttraction,
    /// Lightstall Inquisitor: each opponent exiles a hand card they may play
    /// while exiled for `surcharge` more (stamped as the card's own cost plus
    /// surcharge; X = 0).
    EachOpponentExilesHandCardMayPlay { surcharge: u32 },
    /// Sothera, the Supervoid: each opponent exiles a creature (auto: weakest),
    /// stamped `exiled_with = source` for `CardExiledWithSource`.
    EachOpponentExilesOwnCreature,
    /// Nicol Bolas, Dragon-God's −8 — "Each opponent who doesn't control a
    /// legendary creature or planeswalker loses the game" (CR 104.3a).
    EachOpponentWithoutLegendaryLoses,
    /// Finale of Promise: free-cast up to one graveyard instant (slot 0) and
    /// sorcery (slot 1) with mana value ≤ X, exiled after; at X ≥ 10 copy each
    /// twice.
    FinaleOfPromise,
    /// Feather, the Redeemed: the marked spell is exiled as it resolves and
    /// returns to hand at the next end step.
    MarkExileReturnOnResolve { what: Selector },
    /// Niv-Mizzet Reborn: reveal the top ten; per guild color pair take the first
    /// card whose printed colors are exactly that pair; the rest bottom at random.
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
        /// Only cards matching this may be found ("up to four land cards" —
        /// Realms Uncharted). `None` admits any card.
        #[serde(default)]
        filter: Option<crate::card::SelectionRequirement>,
    },
    /// The Beacon cycle's "shuffle this card into its owner's library": moves the
    /// resolving spell from the stack to the library, before it would reach the
    /// graveyard.
    ShuffleSelfIntoLibrary,
    /// Mirror-Mad Phantasm: the owner shuffles it into their library, then
    /// reveals until a card of its name, which enters; the other revealed cards
    /// go to the graveyard. Nothing unless the source is on the battlefield.
    ShuffleSelfRevealUntilSameNameToBattlefield,
    /// "Return [this spell] to its owner's hand" as part of its own
    /// resolution (Journey to the Oracle's discard rider). Flags the
    /// post-resolution routing the same way `ShuffleSelfIntoLibrary` does.
    ReturnResolvingSpellToHand,
    /// Distribute `total` `kind` counters among tokens created this resolution
    /// (`scratch.last_created_tokens`) — resolution-time `DistributeCounters`,
    /// since fresh tokens can't be cast-time targets (CR 601.2d). UI picks shares
    /// via `ChooseAmount` (last takes the rest); others split evenly. Fractal
    /// Bloom.
    DistributeCountersAmongLastCreated {
        total: Value,
        kind: crate::card::CounterType,
    },
    /// Put the resolving spell `from_top` cards down its owner's library (clamped)
    /// instead of the graveyard, via a transient the post-resolution routing reads
    /// (Approach of the Second Sun).
    PutResolvingSpellInLibraryFromTop(u32),
    /// "Exile this spell, then return it transformed" for the resolving spell,
    /// claimed by the post-resolution routing since the front face isn't a
    /// permanent (Esper Origins). `counter` is placed as it enters.
    PutResolvingSpellOnBattlefieldTransformed {
        #[serde(default)]
        counter: Option<crate::card::CounterType>,
    },
    /// CR 614 — "if target land is tapped for mana, it produces colorless mana
    /// instead", indefinitely (Quarum Trench Gnomes).
    ReplaceTargetLandManaWithColorless { what: Selector },
    /// CR 615 — Silhouette: this turn, prevent damage to it from a source caused
    /// by a spell or ability that targets it.
    PreventTargetingDamageThisTurn { what: Selector },
    /// "Choose a card name. Target opponent reveals `count` cards at random
    /// from their hand, then discards all cards with that name revealed this
    /// way." Nebuchadnezzar.
    NameCardRevealRandomDiscardNamed { who: PlayerRef, count: Value },
    /// Revel in Silence: each resolved player can't cast spells or activate
    /// loyalty abilities for the rest of the turn (`Player.silenced_this_turn`).
    SilencePlayersThisTurn { who: PlayerRef },
    /// "[who] can't cast spells until [their] next turn" (Innocuous
    /// Researcher): the silence carries over each turn boundary until that
    /// player's own turn begins.
    SilencePlayersUntilTheirNextTurn { who: PlayerRef },
    /// "Exile [this spell]" as part of its own resolution (Revel in
    /// Silence). Same flag pattern as `ShuffleSelfIntoLibrary`.
    ExileResolvingSpell,
    /// "…for as long as [this] remains on the battlefield" on a may-play grant
    /// (Intet, the Dreamer): the source's leaves trigger ends the permission
    /// on every card still exiled with it (`exiled_with == source`).
    EndMayPlayOnCardsExiledWithSource,

    /// CR 903 — "Put your commander into your hand from the command zone"
    /// (Command Beacon). With two commanders the controller chooses one.
    /// Does nothing when no commander of `who`'s is in the command zone.
    CommanderToHand { who: PlayerRef },
    /// CR 903 — "Put all commanders from the command zone onto the battlefield
    /// under your control" (Tevesh Szat's −10): every player's commanders in
    /// their command zone, in seat order. Not a cast, so no tax.
    PutCommandersFromCommandZonesOntoBattlefield,
    /// Put a commander you own from the command zone onto the battlefield
    /// (Hellkite Courser) — not a cast (no tax, no cast count); the owner picks
    /// between two; on `Selector::LastMoved`. `haste` grants haste;
    /// `return_at_end_step` adds a CR 603.7a delayed return to the command zone
    /// at the next end step, if it is still there.
    PutCommanderOntoBattlefield {
        who: PlayerRef,
        #[serde(default)]
        haste: bool,
        #[serde(default)]
        return_at_end_step: bool,
    },
    /// CR 903.8 — cast your commander from the command zone without paying its
    /// mana cost (Geode Golem, inside a `MayDo`). Tax is still paid; nothing if
    /// unpayable or not there.
    CastCommanderWithoutPaying,
    /// "The next spell you cast this turn has affinity for artifacts"
    /// (Saheeli, the Gifted's +1): it costs {1} less for each artifact you
    /// control *as you cast it* (CR 702.41a). Lapses with the next spell.
    NextSpellHasAffinityForArtifacts,
    /// "The next **noncreature** spell you cast this turn has affinity for
    /// artifacts" (Don & Raph, Hard Science): as above, but creature spells
    /// cast in between leave it armed (`game/noncreature_affinity.rs`).
    NextNoncreatureSpellHasAffinityForArtifacts,

    // ── Mana ─────────────────────────────────────────────────────────────────
    AddMana { who: PlayerRef, pool: ManaPayload },
    /// Elemental Resonance: add mana equal to a permanent's printed cost —
    /// colored as colored, generic / {C} as colorless, hybrid its first color, X
    /// skipped.
    AddManaEqualToPermanentCost { permanent: Selector },
    /// "Add `amount` mana in any combination of its colors" (Omnath, Locus
    /// of All): `what` is a card in any zone; each pip is one of its colors,
    /// the controller's pick. A colorless card adds nothing.
    AddManaAmongColorsOf { what: Selector, amount: Value },
    /// Invasion of Alara: exile from the top until `count` nonland [filter]
    /// cards; cast one free, take one, bottom the rest at random.
    ExileUntilCastOneTakeOne { count: u32, filter: SelectionRequirement },
    /// CR 500.4 exception (Savage Ventmaw): add these pips, kept through step
    /// and phase ends until cleanup.
    AddManaKeptThisTurn { who: PlayerRef, colors: Vec<Color> },
    /// Like `AddManaKeptThisTurn` but adds `amount` mana of a single `color`
    /// (Neheb, Dreadhorde Champion — "add that much {R}", where the amount is
    /// the number of cards discarded this resolution).
    AddManaKeptThisTurnCount { who: PlayerRef, color: Color, amount: Value },
    /// Klauth, Unrivaled Ancient: `amount` pips of chosen colors, kept until
    /// cleanup and restricted to spells (`SpendRestriction::SpellsOnly`).
    AddManaKeptThisTurnAnyColors { who: PlayerRef, amount: Value },
    /// `amount` pips in any combination of `colors`, each chosen per pip,
    /// kept through step and phase ends until cleanup, spent freely (Grand
    /// Warlord Radha's "any combination of {R} and/or {G}").
    AddManaKeptThisTurnOfColors { who: PlayerRef, colors: Vec<Color>, amount: Value },
    /// "Add that much mana of any one color. Until end of turn, you don't lose
    /// this mana as steps and phases end" (Photon, Mighty Marvel).
    AddManaKeptThisTurnAnyOneColor { who: PlayerRef, amount: Value },
    /// `amount` {C}, kept through step and phase ends this turn, optionally
    /// spend-restricted (Karn, Legacy Reforged — `ArtifactOnly` stands in for
    /// "can't be spent to cast nonartifact spells").
    AddColorlessKeptThisTurn {
        who: PlayerRef,
        amount: Value,
        #[serde(default)]
        restriction: Option<crate::mana::SpendRestriction>,
    },
    /// CR 702.189a — firebending: add `amount` {R} kept until end of combat. A
    /// stack trigger (CR 605.1b), so it can be copied or countered.
    Firebend { amount: Value },
    /// "Add `amount` {R}. Until end of combat, you don't lose this mana as
    /// steps end" (Avatar Roku, Firebender) — firebending's mana without
    /// the CR 702.189b "firebended" event.
    AddRedUntilEndOfCombat { amount: Value },
    /// Engine-appended to a Saga's final chapter ability (never printed):
    /// emits `GameEvent::SagaFinalChapterResolved` as the chapter resolves,
    /// so a countered chapter emits nothing (CR 715.2e).
    SagaFinalChapterResolved,

    // ── Permanent mutations ──────────────────────────────────────────────────
    Destroy { what: Selector },
    /// CR 701.19 — destroy, can't be regenerated (Terminate). Indestructible and
    /// shield counters still apply.
    DestroyNoRegen { what: Selector },
    /// Seeds of Innocence: destroy all [filter], no regeneration; each
    /// controller gains life equal to their permanents' mana value.
    DestroyAllNoRegenGainControllerLifePerManaValue { filter: SelectionRequirement },
    /// Destroy each nonland permanent with mana value `value` (usually charge
    /// counters on this) — Engineered Explosives.
    DestroyEachNonlandWithManaValue { value: Value },
    /// `DestroyEachNonlandWithManaValue` for creatures (Sanguine Praetor, via
    /// `Value::SacrificedManaValue`).
    DestroyEachCreatureWithManaValue { value: Value },
    /// The filtered form of `DestroyEachNonlandWithManaValue`: destroys every
    /// permanent matching `filter` whose mana value equals `value` (Powder
    /// Keg's "each artifact and creature").
    DestroyEachMatchingWithManaValue { filter: SelectionRequirement, value: Value },
    /// "Destroy each [filter] with power `value` or greater", `value` read as
    /// it resolves and the destruction simultaneous (Valiant Endeavor's die).
    DestroyEachMatchingWithPowerAtLeast { filter: SelectionRequirement, value: Value },
    /// "Destroy `what` and all other permanents with the same name" (Wake of
    /// Destruction).
    DestroyAllSharingNameWith { what: Selector },
    /// Expel the Interlopers: choose 0..=`max` (`Decision::ChooseAmount`; auto
    /// 0) and destroy creatures with at least that power.
    ChooseNumberDestroyByPower { max: u32 },
    /// "Choose a number between 0 and `max`. Each player sacrifices that many
    /// [`filter`] of their choice" (By Invitation Only, CR 701.21). The
    /// controller picks at resolution (`AmountKind::EachSacrificesCount`).
    ChooseNumberEachPlayerSacrifices { max: u32, filter: crate::card::SelectionRequirement },
    /// CR 701.15 — a regeneration shield on each resolved permanent: the next
    /// destruction this turn taps it, removes it from combat and heals it
    /// instead (Drudge Skeletons).
    Regenerate { what: Selector },
    /// CR 701.19 — Debt of Loyalty: regenerate `what`; you gain control when
    /// (and only if) the shield is consumed
    /// (`CardInstance.regeneration_control_grant`).
    RegenerateThenGainControl { what: Selector },
    /// CR 701.19 — "it can't be regenerated this turn": existing shields stop
    /// applying and new ones do nothing for the rest of the turn. Rage of
    /// Purphoros, the Terror-style removal riders.
    CantBeRegeneratedThisTurn { what: Selector },
    /// "If [it] would die this turn, exile it instead": a turn-long CR 614 death
    /// replacement (finality-counter shape), so later deaths that turn are caught
    /// too. Wilt in the Heat.
    ExileIfWouldDieThisTurn { what: Selector },
    /// CR 614 — Cosmic Intervention: this turn your dying permanents (later
    /// arrivals included) are exiled instead, recorded against the source for
    /// [`Effect::ReturnCardsExiledInsteadBySource`].
    ExileYourDyingPermanentsThisTurn,
    /// "Return it to the battlefield under its owner's control": each card
    /// `ExileYourDyingPermanentsThisTurn` exiled for this source and still
    /// in exile.
    ReturnCardsExiledInsteadBySource,
    /// CR 614 — "if it would leave the battlefield, exile it instead": a
    /// replacement bound to each resolved permanent, lapsing when it leaves (CR
    /// 400.7). Whip of Erebos, Gruesome Encore.
    ExileIfLeavesBattlefield { what: Selector },
    /// Exile `what` linked to the object `link` names (`exiled_with`), for its
    /// own `CardExiledWithSource` (Timothar's Bat).
    ExileLinkedTo { what: Selector, link: Selector },
    /// "Gains flashback until end of turn equal to its mana cost": sets
    /// `granted_flashback_eot` on each resolved graveyard card, castable via the
    /// normal flashback path. SOS "Flashback".
    GrantFlashbackThisTurn { what: Selector },
    /// `GrantFlashbackThisTurn` at a printed flashback cost ("gains flashback
    /// {2}{R}{G} until end of turn" — The Fugitive Doctor).
    GrantFlashbackWithCostThisTurn { what: Selector, cost: crate::mana::ManaCost },
    /// "Target instant or sorcery card in your graveyard gains jump-start
    /// until end of turn" (Filigree Racer, CR 702.133): stamps
    /// `granted_jumpstart_eot`, cleared at cleanup.
    GrantJumpStartThisTurn { what: Selector },
    /// CR 702.138 — "gains escape until end of turn; cost its mana cost plus
    /// exiling `exile_count` other graveyard cards" (Desdemona, Freedom's Edge).
    /// In `GameState::granted_escape_eot`, read first by `effective_escape_grant`;
    /// cleared at cleanup.
    GrantEscapeThisTurn { what: Selector, exile_count: u32 },
    /// As `GrantEscapeThisTurn` with a printed escape cost ("each creature
    /// card in your graveyard gains 'Escape—{3}{B}, exile four other cards'"
    /// — The Grim Captain's Locker).
    GrantEscapeWithCostThisTurn { what: Selector, cost: crate::mana::ManaCost, exile_count: u32 },
    /// CR 702.88 — "gains embalm until end of turn, cost its mana cost"
    /// (Cursecloth Wrappings): a real embalm activation in
    /// `granted_activated_eot`, cleared at cleanup.
    GrantEmbalmThisTurn { what: Selector },
    /// CR 702.141 — "[card] gains encore until end of turn. The encore cost
    /// is equal to its mana cost" (Araumi of the Dead Tide).
    GrantEncoreThisTurn { what: Selector },
    /// CR 702.180 — "gains harmonize until end of turn, cost its mana cost"
    /// (Songcrafter Mage): `granted_harmonize_eot`, castable via the normal
    /// Harmonize path; cleared at cleanup.
    GrantHarmonizeThisTurn { what: Selector },
    /// "Gains miracle `cost` until end of turn": a turn-long `may_play_until` plus
    /// `granted_alt_cast_cost_eot` of `cost` (Lorehold, the Historian).
    GrantMiracle { what: Selector, cost: crate::mana::ManaCost },
    /// `GrantMiracle` whose cost is each card's own mana cost reduced by
    /// `{reduce}` generic (Aminatou, Veil Piercer — "its miracle cost is equal
    /// to its mana cost reduced by {4}").
    GrantMiracleReduced { what: Selector, reduce: u32 },
    Exile   { what: Selector },
    /// Bloomburrow Enduring: "when this dies, if it was a creature, return it.
    /// It's an enchantment." Returns the source from the graveyard without the
    /// Creature type, so the gate can't loop. No-op unless it's a creature card
    /// in a graveyard.
    ReturnSelfAsEnchantment,
    /// Otherworldly Escort: "if it's not a `unless`, return it with `amount`
    /// `kind` counters; it's a `types`." The retype is on the card's definition,
    /// so the gate self-limits like `ReturnSelfAsEnchantment`.
    ReturnSelfRetypedWithCounters {
        unless: crate::card::CreatureType,
        types: Vec<crate::card::CreatureType>,
        kind: crate::card::CounterType,
        amount: u32,
    },
    /// "When this dies, return it tapped with `amount` `kind` counters"
    /// (Unstoppable Slasher). No-op unless in a graveyard.
    ReturnSelfTappedWithCounters { kind: crate::card::CounterType, amount: u32 },
    /// Return the source tapped from its owner's graveyard — counterless
    /// `ReturnSelfTappedWithCounters` (Fake Your Own Death).
    ReturnSelfTapped,
    /// "Return it to the battlefield under its owner's control" (untapped) —
    /// the untapped sibling of `ReturnSelfTapped` (Presumed Dead's granted
    /// die-trigger). No-op if the source isn't in a graveyard.
    ReturnSelf,
    /// CR 702.171 — flicker the source and up to one creature that saddled it
    /// this turn (`CardInstance.saddled_by`). Fortune, Loyal Steed. Bots take one
    /// saddler; no-op off the battlefield.
    ExileAndReturnSelfWithSaddler,
    /// Bronzehide Lion: return the dead source as its `back_face` Aura on your
    /// strongest creature. No-op without a graveyard card, back face or creature.
    ReturnSelfTransformedAttached,
    /// "Return this card to the battlefield attached to [slot-0 target]"
    /// (Gift of Immortality's end-step re-attach). No-op if the source isn't
    /// in a graveyard or the target isn't a battlefield permanent.
    ReturnSelfAttachedToTarget,
    /// "Return this card from your graveyard to the battlefield attached to
    /// that creature" — the trigger-source sibling of
    /// [`Effect::ReturnSelfAttachedToTarget`] (the Scourge Dragon Auras).
    ReturnSelfAttachedToTrigger,
    /// "[Chooser] chooses a [filter] permanent; return this card attached to it"
    /// (Necrotic Plague). A source orphaned on the battlefield is re-attached in
    /// place (Steam Vines). `filter` is read from the chooser's seat. No-op with
    /// no candidate.
    ReturnSelfAttachedToChoiceOf { chooser: PlayerRef, filter: crate::card::SelectionRequirement },
    /// "Return the top creature card of `who`'s graveyard to the battlefield."
    /// Top = most recently put into the graveyard (Mistmoon Griffin). No-op if
    /// the graveyard holds no creature card.
    ReturnTopCreatureFromGraveyard { who: PlayerRef },
    /// "Look at the top `count` cards of your library, then put one of them on
    /// the bottom" (Preferred Selection's declined branch). The rest stay on
    /// top in order; an auto seat buries the priciest card.
    LookTopPutOneOnBottom { count: Value },
    /// CR 712 — transform each target in place, keeping counters, tapped state
    /// and attachments (CR 712.12); emits `Transforms` per permanent.
    Transform { what: Selector },
    /// CR 709.5 — unlock one locked door (left first) of each resolved Room,
    /// firing its unlock triggers and the Eerie `RoomFullyUnlocked` event.
    /// Ghostly Keybearer.
    UnlockRoomDoor { what: Selector },
    /// CR 709.5c — "Lock or unlock a door of target Room you control" (Marina
    /// Vendrell). Unlocks a still-locked door when there is one; otherwise
    /// re-locks the right door so it can be unlocked (and re-triggered) later.
    LockOrUnlockRoomDoor { what: Selector },
    /// CR 702.112 — mark each matching permanent as renowned (sets
    /// `CardInstance.renowned`). Paired with the Renown counter add so the
    /// once-only gate keys off the real flag, not a counter heuristic.
    BecomeRenowned { what: Selector },
    /// CR 701.64a — "Harness [this permanent]": each matching permanent that
    /// isn't harnessed becomes harnessed (`CardInstance.harnessed`).
    Harness { what: Selector },
    /// CR 710.2 — flip each flip card to its bottom face in place; one-way,
    /// no-op if already flipped. Emits `Flipped`.
    Flip { what: Selector },
    /// CR 701.42 Meld: if the controller owns and controls the source and a
    /// `partner`, exile both and put `into` onto the battlefield. The parts ride
    /// `CardInstance.meld_parts`, so it leaves as both cards (CR 713.8/712.14).
    /// Otherwise no-op (701.37b).
    Meld {
        partner: String,
        into: String,
        /// "If [this] and [partner] are attacking … meld them … It enters
        /// tapped and attacking" (Mishra, Claimed by Gix): both halves must be
        /// attacking, and the melded permanent joins combat (CR 508.4).
        #[serde(default)]
        attacking: bool,
    },
    /// Urza, Planeswalker's +2: [filter] spells cost {amount} less this turn
    /// (`Player.turn_spell_discounts`, cleared at cleanup, CR 514.2).
    SpellsCostLessThisTurn { filter: SelectionRequirement, amount: u32 },
    /// `SpellsCostLessThisTurn` whose amount is a `Value` fixed as it resolves
    /// (Rowan, Scion of War: "X is the amount of life you lost this turn",
    /// 2023-09-01 ruling).
    SpellsCostLessThisTurnByValue { filter: SelectionRequirement, amount: Value },
    /// Cheering Fanatic: name a card (the priciest nonland in hand); spells with
    /// that name cost {amount} less this turn, for everyone.
    NameSpellsCostLessThisTurn { amount: u32 },
    /// "Until your next turn, [filter] spells `who` casts cost {amount} less
    /// to cast" (Will Kenrith's −2) — the discount mirror of
    /// `SpellTaxUntilYourNextTurn`, expiring at the controller's untap.
    SpellDiscountUntilYourNextTurn { who: PlayerRef, amount: u32, filter: SelectionRequirement },
    /// "Face-down spells you cast this turn cost {amount} less to cast"
    /// (Goblin Maskmaker). Bumps `Player.face_down_discount_this_turn`, read
    /// by `face_down_cast_cost` and cleared at cleanup (CR 514.2).
    FaceDownSpellsCostLessThisTurn { amount: u32 },
    /// Panoptic Projektor: the next face-down creature spell this turn costs
    /// {amount} less (`Player.next_face_down_discount_this_turn`).
    NextFaceDownSpellCostsLessThisTurn { amount: u32 },
    /// "That spell gains [keywords]" — grants keywords to a spell on the
    /// stack for as long as it's there (Judith, Carnage Connoisseur).
    /// Recorded in `GameState.spell_keyword_grants`.
    GrantKeywordsToSpell { what: Selector, keywords: Vec<Keyword> },
    /// Cosima, God of the Voyage's exiled landfall, run from exile: put a
    /// voyage counter on the source, or (`home`) return it to the battlefield
    /// with X +1/+1 counters and draw X, X being its voyage counters.
    Voyage { home: bool },
    /// "Exile [what] and the top `depth` cards of your library in a face-down
    /// pile, shuffle that pile, and put it back on top" (Triumph of Saint
    /// Katherine) — only while [what] is still in its owner's graveyard.
    ShuffleIntoTopPile { what: Selector, depth: Value },
    /// "You may cast [what] by paying `extra` in addition to its other costs"
    /// (Ogre Battlecaster): its own cost plus `extra`, taxes and discounts on
    /// top; `exile_after` exiles it instead of the graveyard.
    CastImmediatePayingExtra {
        what: Selector,
        source_zone: crate::card::Zone,
        extra: crate::mana::ManaCost,
        exile_after: bool,
    },
    /// "Copy it. You may cast the copy by paying `cost` rather than its mana
    /// cost" (Blue Mage's Cane) — an alternative cost for a copy cast as this
    /// resolves; taxes and discounts still apply.
    CastCopyForCost { what: Selector, source_zone: crate::card::Zone, cost: crate::mana::ManaCost },
    /// "That creature enters with N additional [kind] counters" on a spell on the
    /// stack, or on a card a following move puts onto the battlefield (Grave
    /// Endeavor): stamps the object's `pending_etb_counters`.
    SpellEntersWithCounters { what: Selector, kind: crate::card::CounterType, amount: Value },
    /// "It gains [keyword]" on a spell still on the stack — stamps the
    /// keyword onto the spell's card, so it holds as the permanent enters
    /// (Bloodlord of Vaasgoth's and Twins of Discord's granted bloodthirst).
    SpellGainsKeyword { what: Selector, keyword: crate::card::Keyword },
    /// "Put that card into your hand instead of into your graveyard as it
    /// resolves" on a spell still on the stack — the buyback routing without
    /// the cost (CR 702.27; Soulfire Grand Master).
    SpellReturnsToHandOnResolve { what: Selector },
    /// "It gains '[trigger]'" on a spell still on the stack — stamped onto the
    /// spell's card like `SpellGainsKeyword`, so the permanent it becomes has
    /// it (Rivaz of the Claw's "When this creature dies, exile it.").
    SpellGainsTrigger { what: Selector, trigger: Box<crate::card::TriggeredAbility> },
    /// "Exile target [permanent], then search its owner's graveyard, hand, and
    /// library for any number of cards with the same name and exile them"
    /// (Crumble to Dust). `what` is the anchor (slot 0).
    ExileSameNameAsTarget { what: Selector },
    /// Exile the targets stamped `exiled_with = source`, for counts like
    /// `Value::DistinctCardTypesExiledWith` (Keen-Eyed Curator).
    ExileTaggedWithSource { what: Selector },
    /// CR 702.75 Hideaway N: look at the top `count`, exile one face down with
    /// `exiled_with = source` (auto: the priciest), bottom the rest at random.
    /// Played later via `CastWithoutPayingImmediate { what:
    /// CardExiledWithSource }`.
    Hideaway { count: Value },
    /// "Exile any number of target cards from graveyards" (Devious Cover-Up): a
    /// `Decision::ChooseCards` subset of `filter` matches. AutoDecider exiles
    /// none; the bot exiles opponents' cards.
    ExileAnyNumberFromGraveyards { filter: crate::card::SelectionRequirement },
    /// CR 603.7 — "you may exile one or more `filter` cards from your graveyard.
    /// When you do, `then`." The exiled cards are `Selector::LastMoved`
    /// (Specter of Mortality's -X/-X); declining skips `then`.
    MayExileFromYourGraveyard {
        filter: crate::card::SelectionRequirement,
        /// At most this many ("you may exile a creature card" — Creeping Inn:
        /// `Some(1)`); `None` is "one or more".
        #[serde(default)]
        max: Option<u32>,
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
    /// Living End: each player exiles graveyard creatures, sacrifices their
    /// creatures, then the exiled cards enter under their owners.
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
    /// Mindleech Mass: look at the player's hand and cast one card free.
    /// `filter` narrows it (Mindclaw Shaman); `None` = any nonland.
    LookAtHandCastFree {
        who: Selector,
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },
    /// Reroute — change the target of the resolved activated ability on the
    /// stack to a new legal one (CR 115.7b). Single-target abilities only.
    ChangeTargetOfAbility { what: Selector },
    /// Aetherplasm — return the source to its owner's hand, then put a
    /// creature card from the controller's hand onto the battlefield blocking
    /// the creature the source was blocking (CR 509.1 — a late blocker).
    ReturnSelfDeployBlocker,
    /// Ink-Treader Nephilim: copy the resolved spell once per other creature it
    /// could target, each copy aimed at a different one. `casters_creatures`:
    /// only the caster's creatures (Mirrorwing Dragon).
    CopySpellForEachOtherLegalCreature {
        what: Selector,
        #[serde(default)]
        casters_creatures: bool,
    },
    /// Copy the spell `what` resolves to once, aimed at the object `target`
    /// resolves to — Frontline Heroism's "then copy that spell. The copy
    /// targets that token."
    CopySpellTargeting { what: Selector, target: Selector },
    /// Mimeofacture — search the resolved permanent's controller's library for
    /// a card with that permanent's name and put it onto the battlefield under
    /// *your* control; then that player shuffles.
    SearchOpponentLibraryForSameName { what: Selector },
    /// Verdant Succession: search `who`'s library for a card named like `what`
    /// (may have left the battlefield), put it onto the battlefield, shuffle.
    SearchSameNameToBattlefield { who: PlayerRef, what: Selector },
    /// Haunting Echoes: exile every library card named like a card exiled
    /// earlier this resolution, then shuffle.
    ExileLibraryCardsNamedLikeExiledThisResolution { who: PlayerRef },
    /// Development — "create a 3/1 red Elemental token unless any opponent has
    /// you draw a card", `times` times. Each iteration asks an opponent.
    TokenUnlessOpponentLetsYouDraw { token: std::sync::Arc<TokenDefinition>, times: u32 },
    /// Sunforger: search for a `filter` card, cast it free, shuffle.
    /// `include_hand`: hand too, shuffling only on a library find (Aether
    /// Searcher).
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
    /// Memory Leak: exile a `filter` card from `who`'s graveyard or hand, your
    /// choice. Headless: priciest, and from your own cards one with an activated
    /// ability first.
    ExileChosenFromHandOrGraveyard { who: PlayerRef, filter: SelectionRequirement },
    /// "Target player reveals the top `reveal`; you put `pick` of them into their
    /// graveyard, the rest back in any order" (Bamboozle; Balshan Beguiler at
    /// (2, 1)). Auto-picks the priciest.
    RevealTopChooseToGraveyard { who: PlayerRef, reveal: Value, pick: Value },
    /// Aura Graft: gain control of the Aura and move it to another permanent it
    /// can enchant (auto: your cheapest legal one, else any).
    GainControlAndReattachAura { what: Selector },
    /// "Mill a card. For each coloured mana symbol in the milled card's mana
    /// cost, add one mana of that colour." Charmed Pendant.
    MillAddManaForColoredSymbols { who: PlayerRef },
    /// Decaying Soil: exile `count` `filter` cards from `who`'s graveyard,
    /// mandatory (cheapest; fewer if short).
    ExileFromGraveyard { who: PlayerRef, count: Value, filter: SelectionRequirement },
    /// "Exile the *bottom* card of `who`'s graveyard" (Phyrexian Furnace). The
    /// graveyard is ordered, so this is index 0 — the oldest card, not the
    /// freshest one `ExileFromGraveyard`'s auto-pick would take.
    ExileBottomOfGraveyard { who: PlayerRef },
    /// CR 701.38 — an option vote. Starting with the controller and proceeding
    /// in turn order, each player votes for one of `options`; how the tally is
    /// spent is `tally`'s business. Untargeted, so it ignores hexproof/shroud.
    Vote { options: Vec<VoteOption>, tally: VoteTally },
    /// CR 701.38 — "you choose how each player votes this turn" (Illusion of
    /// Choice). Sets `GameState.vote_controller_this_turn` to the resolved
    /// player; each vote this turn is answered by them on the voter's behalf.
    ControlVotesThisTurn { who: PlayerRef },
    /// CR 701.38 Will of the council: from the controller, each player votes for
    /// a `filter` permanent (read relative to the controller); the most-voted
    /// (ties included) are exiled. AutoDecider votes first-legal. Not targeting
    /// (CR 701.31c), so hexproof doesn't stop it. Council's Judgment.
    WillOfTheCouncilExile { filter: SelectionRequirement },
    /// General [`Effect::WillOfTheCouncilExile`]: votes over `candidates` (any
    /// zone); every card tied for most moves to `to` (Custodi Squire).
    WillOfTheCouncilOnCards { candidates: Selector, to: ZoneDest },
    /// "Put the bottom card of your library into your graveyard. If it's a
    /// creature card with power less than or equal to `max_power`, put it onto
    /// the battlefield" (Grenzo, Dungeon Warden).
    BottomCardToGraveyardThenDeploy { max_power: Value },
    /// "Starting with the next opponent, each player chooses a [filter]. Destroy
    /// the chosen" (Sadistic Shell Game); "you" in the filter is the controller.
    /// `starting_with_you`: the caster picks first and later seats see it (CR
    /// 101.4) — The Horus Heresy.
    EachPlayerChoosesToDestroy {
        filter: SelectionRequirement,
        #[serde(default)]
        starting_with_you: bool,
        /// "Each player **may** choose" (Druid of Purification): a seat may
        /// decline.
        #[serde(default)]
        may: bool,
    },
    /// The Abyss: `who` destroys one of their `filter` permanents (`no_regen`:
    /// can't be regenerated). "You" in the filter is `who`.
    PlayerChoosesToDestroy {
        who: PlayerRef,
        filter: SelectionRequirement,
        #[serde(default)]
        no_regen: bool,
    },
    /// "Starting with you, each player chooses one permanent matching each of
    /// `filters` from among those controlled by the player to their left.
    /// Destroy each permanent chosen this way." Grenzo's Rebuttal.
    EachPlayerDestroysChosenFromLeftNeighbor { filters: Vec<SelectionRequirement> },
    /// Exile each resolved object with `exiled_with = source`
    /// (`Selector::CardExiledWithSource`) but no leaves-the-battlefield return —
    /// the scheduling effect owns the return (Legion's Initiative).
    ExileLinked { what: Selector },
    /// Stamp `exiled_with = source` on `LastMoved` cards in exile for
    /// `CardsExiledWithSourceCount` (Unlicensed Hearse).
    LinkLastMovedExilesToSource,
    /// Search the City: if an exiled card shares the played card's name, return
    /// one; when the pile empties, sacrifice this and take an extra turn.
    SearchTheCityReturn,
    /// CR 603.6e — "exile [what] until [this] leaves the battlefield", linked to
    /// the source; when it leaves, the cards go to `return_to` (Oblivion Ring
    /// to the battlefield, Tidehollow Sculler to hand).
    ExileUntilSourceLeaves {
        what: Selector,
        return_to: crate::card::ExileReturnZone,
    },
    /// Mysterious Limousine: after an `ExileUntilSourceLeaves`, if a creature was
    /// exiled, every other card still linked to the source (bar this
    /// resolution's `LastMoved`) returns as its link says (CR 610.3).
    ReturnOtherLinkedExiles,
    /// CR 603.6e — exile any number of your other `filter` permanents until this
    /// leaves (Lumbering Battlement), each linked like `ExileUntilSourceLeaves`.
    /// `shelter` (Glorious Protector): headless exiles only what the stack
    /// threatens; otherwise everything.
    ExileAnyNumberUntilSourceLeaves {
        filter: SelectionRequirement,
        #[serde(default)]
        shelter: bool,
    },
    /// CR 603.6e — "each player chooses [count] [filter] they control; exile them
    /// until this leaves" (Foreboding Steamboat): APNAP picks (all if `count` or
    /// fewer), exiled at once, linked to the source.
    EachPlayerExilesChosenUntilSourceLeaves { count: Value, filter: SelectionRequirement },
    /// CR 726 — Palace Jailer: exile until an opponent becomes the monarch. The
    /// card carries a `monarch_guard` for the controller and returns when the
    /// monarchy leaves them, not when the source leaves.
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
    /// Illusionist's Bracers: copy the activated ability that triggered this. The
    /// source's topmost `activated` stack item is always legal (mana abilities
    /// never reach the stack, CR 707.10). The copy may retarget every slot,
    /// original offered first.
    CopyActivatedAbilityMayChooseTargets,
    /// Agrus Kos, Eternal Soldier: copy the ability targeting only the source
    /// once per other creature you control it could target (CR 707.10).
    CopyAbilityTargetingSourceForEachOtherCreature,
    /// Semester's End: exile, then return at the next end step with an extra
    /// +1/+1 (creature) or loyalty (planeswalker) counter.
    ExileReturnNextEndStep { what: Selector },
    /// Exile each resolved permanent, returning it under its owner's control at
    /// the controller's next upkeep (Kaya, Ghost Assassin's 0).
    ExileReturnAtYourNextUpkeep { what: Selector },
    /// Exile each resolved permanent and return it under its owner's control at
    /// the next end step, no counter (Voidwalk) — the plain-flicker sibling of
    /// `ExileReturnNextEndStep`.
    ExileReturnToOwnerNextEndStep {
        what: Selector,
        /// The permanent returns tapped (Mystifying Maze).
        #[serde(default)]
        tapped: bool,
    },
    /// "You may exile this card. If you do, [body]": the source moves from
    /// wherever it is (a death trigger finds it in the graveyard) to exile, then
    /// `body` runs (Academy Rector).
    MayExileSelfThen { body: Box<Effect> },
    /// Scrying Glass — "choose a number greater than 0 and a color; `who`
    /// reveals their hand; if they reveal exactly that many cards of that
    /// colour, draw a card." The guess is made by the effect's controller.
    GuessColorCountInHand { who: PlayerRef, max: u32 },
    /// "Attach target Aura card from a graveyard to this creature" — the Aura
    /// returns to the battlefield already attached (Iridescent Drake).
    AttachAuraFromGraveyardTo { aura: Selector, host: Selector },
    /// CR 603.10a — Cass, Hand of Vengeance: return the dying subject's Auras
    /// from your graveyard attached to `host`, and attach its Equipment to it
    /// (`GameState.auras_at_death`). Takes every one it may: Aura cards it owns
    /// still in the graveyard, Equipment it controls still on the battlefield.
    ReturnDyingSubjectAttachmentsTo { host: Selector },
    /// Zack Fair: attach the first Equipment from the source's leaves record
    /// (`auras_at_death`) still controlled by you to `host`.
    AttachSourcesFormerEquipmentTo { host: Selector },
    /// CR 707.2 + 303.4f — per `hosts` permanent, a token copy of Aura `source`
    /// (or its LKI) attached to it (Three Dog, Galaxy News DJ).
    CreateTokenCopyOfAttachedToEach { source: Selector, hosts: Selector },
    /// Vault 21: House Gambit: reveal up to `max` nonland hand cards; a Treasure
    /// per card sharing a mana value with another revealed one. Reveals whole
    /// equal-value groups, largest first, while two of a group still fit.
    TreasurePerPairedManaValueInHand { max: u32 },
    /// CR 701.3 — "Attach [this] to `host`" (The Aetherspark's +1). Moves the
    /// source's attachment without paying an equip cost; a `host` that resolves
    /// to nothing leaves it where it is.
    AttachSourceTo { host: Selector },
    /// "Until end of turn, whenever a player taps a `land` for mana, that
    /// player adds an additional `extra`" (Bubbling Muck). The turn-scoped
    /// sibling of `StaticEffect::ExtraManaOnLandTap`.
    ExtraManaOnLandTapThisTurn { land: crate::card::LandType, extra: crate::mana::Color },
    /// "You may exile this. If you do, return it at the beginning of your next
    /// upkeep. It gains haste" (Obzedat, Ghost Council; CR 603.4).
    MayExileSelfReturnNextUpkeepHaste,
    /// "Return this card to the battlefield tapped under its owner's control
    /// at the beginning of their next upkeep" (Phytotitan). Fired from a
    /// dies-trigger; registers a `DelayedKind::YourNextUpkeep` return.
    ReturnSelfAtNextUpkeepTapped,
    /// CR 702.55 Haunt: exile the source haunting a creature (auto: an
    /// opponent's; choice is a UI follow-up) with a
    /// `DelayedKind::WhenHauntedCreatureDies` trigger running `body`.
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
    /// Prairie Dog: this turn, +1/+1 counters on your creatures get one extra
    /// (`Player.extra_plus_one_counters_this_turn`).
    GrantExtraPlusOneCountersThisTurn { who: PlayerRef },
    /// Brad Boimler: this turn, counters of any kind put on `who`'s permanents
    /// get one extra each (`Player.extra_counters_this_turn`).
    GrantExtraCountersThisTurn { who: PlayerRef },
    /// Combine Guildmage: this turn your creatures enter with an extra +1/+1
    /// counter (stacks; `Player.extra_etb_p1p1_counters_this_turn`). Unlike
    /// `GrantExtraPlusOneCountersThisTurn`, which amplifies placement.
    CreaturesEnterWithExtraCounterThisTurn { who: PlayerRef },
    /// "The next time one or more enchantment creatures you control enter
    /// this turn, each enters with `amount` additional +1/+1 counters"
    /// (Storyweave) — one batch, then gone (`ColdState.ench_creature_etb_bonus`).
    NextEnchantmentCreaturesEnterWithCountersThisTurn { amount: u32 },
    /// As `CreaturesEnterWithExtraCounterThisTurn`, "until your next turn"
    /// (Arlinn, the Pack's Hope).
    CreaturesEnterWithExtraCounterUntilYourNextTurn { who: PlayerRef },
    /// CR 509.1h — "target unblocked attacking creature becomes blocked."
    /// Marks the attacker blocked with no blockers assigned, so (absent
    /// trample) it deals no combat damage. Curtain of Light.
    BecomeBlocked { what: Selector },
    /// Sweep (CR 207.2c): return any number of your `filter` permanents to hand;
    /// the count is `Value::PermanentsReturnedThisEffect` for a later step (Barrel
    /// Down Sokenzan).
    ReturnAnyNumberToHand { filter: SelectionRequirement },
    /// Footbottom Feast: put any number of graveyard [filter] cards on top, in
    /// mana-value order with the priciest drawn first.
    PutAnyNumberFromGraveyardOnTop { filter: SelectionRequirement },
    /// Orphans of the Wheat: tap any number of your `filter` permanents; this
    /// gets +`power`/+`toughness` per tapped.
    TapAnyNumberThenPumpPerTapped {
        filter: SelectionRequirement,
        power: i32,
        toughness: i32,
    },
    /// "You may tap any number of untapped `filter` you control. If you do, put
    /// a `counter` counter on each of those" (Urge to Feed). The counter-payout
    /// sibling of `TapAnyNumberThenPumpPerTapped`.
    TapAnyNumberThenCounters { filter: SelectionRequirement, counter: crate::card::CounterType },
    /// "You may tap any number of untapped `filter` you control. Draw a card
    /// for each one tapped this way" (Guild Summit).
    TapAnyNumberThenDraw { filter: SelectionRequirement },
    /// "Tap any number of untapped `filter` you control. [body] for each one
    /// tapped this way" (Devout Invocation's Angels): `body` runs once with
    /// `Value::XFromCost` the number tapped. Nothing runs when none are.
    TapAnyNumberThen { filter: SelectionRequirement, body: Box<Effect> },
    /// Entrancing Lyre — tap `what` and lock it from untapping for as long as
    /// the source permanent stays tapped (`CardInstance.untap_locked_by`).
    TapAndUntapLock { what: Selector },
    /// Somnophore: tap `what` and keep it from untapping while the source remains
    /// (`CardInstance.untap_locked_while_present`); `while_you_control`: while you
    /// control it (Shipbreaker Kraken).
    TapAndLockWhileSourcePresent {
        what: Selector,
        #[serde(default)]
        while_you_control: bool,
    },
    /// CR 702.158d — "choose a sector, then [body] each creature in it."
    /// The controller picks alpha/beta/gamma; `body` reads the picks via
    /// `Selector::CreaturesInChosenSector` (Space Beleren's −1 and −5).
    ChooseSector { body: Box<Effect> },
    /// CR 702.158d — creatures can be blocked this turn only by creatures in
    /// the same sector (Space Beleren's +1).
    SectorBlockLockThisTurn,
    /// "Tap each creature that was blocked by [what] this turn; those
    /// creatures don't untap during their controllers' next untap steps"
    /// (Triton Tactics), reading the game-level `blocks_declared_this_turn` log.
    TapBlockedByAndSkipUntap { what: Selector },
    /// CR 506.4 — remove each target from combat (attacker's blocks released,
    /// blocker unassigned); it stays on the battlefield.
    RemoveFromCombat { what: Selector },
    /// CR 508.1a — "that creature can't attack during its controller's next
    /// turn" (Wall of Dust). Arms `CardInstance.attack_ban`, which the
    /// controller's untap step promotes and that turn's cleanup clears.
    CantAttackNextTurn { what: Selector },
    /// CR 508.1a — "[creatures] can't attack this turn" (Festival). Arms the
    /// ban live for the current turn rather than the next one.
    CantAttackThisTurn { what: Selector },
    /// "This turn and next turn, creatures can't attack, and players and
    /// permanents can't be the targets of spells or activated abilities"
    /// (Peace Talks). Sets `GameState.truce_until_turn`.
    TruceThisTurnAndNext,
    /// "You may repeat this process any number of times" (Forbidden Ritual) —
    /// run `body`, then keep asking whether to run it again, up to `max`
    /// iterations. The costless sibling of `MayPayRepeatedly`.
    MayRepeat { description: String, body: Box<Effect>, max: u32 },
    /// Pygmy Hippo: the defender taps all their lands for mana and loses it; this
    /// assigns no combat damage and you add that much {C} at your next main
    /// phase this turn.
    DrainDefendersLandsForManaNextMain,
    /// "Until end of turn, if you tap a land you control for mana, it produces
    /// [color] instead of any other type" (Deep Water). The turn-scoped,
    /// controller-scoped sibling of `StaticEffect::LandsProduceColorInstead`.
    YourLandsProduceColorThisTurn(crate::mana::Color),
    /// Hall of Gemstone: a chosen color for every land tapped for mana this turn,
    /// for every seat — global `YourLandsProduceColorThisTurn`.
    AllLandsProduceChosenColorThisTurn { chooser: PlayerRef },
    /// "Each player may discard up to `max` cards. This deals damage to each
    /// player equal to `max` minus the number they discarded" (Mind Bomb).
    EachPlayerMayDiscardUpToThenDamage { max: u32 },
    /// Borderland Explorer: APNAP, each player may discard; those who did may
    /// tutor a basic land to hand.
    EachPlayerMayDiscardThenTutorBasic,
    /// CR 701.3c — move the targeted Aura to another permanent of that type
    /// (Enchantment Alteration); illegal for its enchant filter, nothing happens.
    ReattachTargetAura { aura: Selector, to: Selector },
    /// CR 702.26 — phase out each resolved permanent (into `GameState.phased_out`)
    /// until its controller's next untap step (Vodalian Illusionist).
    /// `until_source_leaves`: they return only when the source leaves, which gains
    /// a time counter per permanent (Out of Time).
    PhaseOut {
        what: Selector,
        #[serde(default)]
        until_source_leaves: bool,
    },
    /// CR 702.26 — after `PhaseOut { until_source_leaves: true }` printed "for as
    /// long as you control [this]" (The Phasing of Zhalfir): losing control phases
    /// them in too. Not for holds without that clause (Out of Time).
    HoldPhasedOutWhileYouControlSource,
    /// CR 702.26 — "phase out; until the end of your next turn, they can't phase
    /// in" (Teferi, Timeless Voyager's −8): `PhaseOut` plus a
    /// `ColdState::phase_in_locks` row honoured until the controller's next turn
    /// has begun (so ends at the untap after it).
    PhaseOutUntilYourNextTurnEnds { what: Selector },
    /// "That player exiles a card at random from their hand. The player may
    /// play that card this turn" (Elkin Lair). The exile is stamped
    /// `exiled_with = source`, so a delayed trigger can bin whatever is left.
    ExileRandomFromHandMayPlayThisTurn { who: PlayerRef },
    /// CR 502.1 — Undiscovered Paradise: return `what` to hand during your next
    /// untap step, a turn-based action (an armed flag), not a trigger.
    ReturnToHandAtYourNextUntapStep { what: Selector },
    /// CR 702.26 — "simultaneously, all phased-out [filter] phase in and all
    /// [filter] with phasing phase out" (Time and Tide). Both sets are read
    /// before either moves, so a phaser that just returned stays.
    SwapPhasedState { filter: SelectionRequirement },
    /// "If the top card of `who`'s graveyard matches `filter`, put it on top of
    /// that player's library" (Guiding Spirit).
    TopOfGraveyardToLibraryTop { who: PlayerRef, filter: SelectionRequirement },
    /// "Look at the top card of `who`'s library. If it matches `filter`, you
    /// may pay `life` life. If you do, put it into that player's graveyard"
    /// (Wand of Denial).
    LookTopMayPayLifeToBin { who: PlayerRef, filter: SelectionRequirement, life: u32 },
    /// Untap each resolved permanent; `up_to` caps it at N (Frantic Search),
    /// `None` = all. Over the cap, the first `up_to` in resolution order;
    /// auto-resolution prefers the priciest lands.
    Untap   { what: Selector, #[serde(default)] up_to: Option<Value> },
    /// "Simultaneously untap all tapped [what] and tap all untapped [what]"
    /// (Breaking Wave). A sequenced Untap-then-Tap would re-tap everything it
    /// just untapped, so the swap is read once and applied in one pass.
    SwapTappedState { what: Selector },
    /// Give a temporary +P/+T bonus.
    PumpPT  { what: Selector, power: Value, toughness: Value, duration: Duration },
    /// Exponential Growth: double the creature's power `times` times for
    /// `duration` (adds `power * (2^times - 1)`). `times` ≤ 0 is a no-op.
    DoublePower { what: Selector, times: Value, duration: Duration },
    /// CR 701.10 — double the `kind` counters on each resolved permanent (add as
    /// many as it has), through counter doublers (Kalonian Hydra).
    DoubleCountersOnEach { what: Selector, kind: crate::card::CounterType },
    /// CR 701.10 — double every counter kind on each permanent (N → 2N), through
    /// doublers. Vorel of the Hull Clade.
    DoubleAllCountersOn { what: Selector },
    /// Sets base P/T through a layer-7b `Modification::SetPowerToughness`, unlike
    /// `PumpPT`'s bonus fields (Square Up). Counters and pumps still apply on top
    /// (CR 613.7c / 613.7f): a +1/+1 counter makes a 0/4 a 1/5.
    SetBasePT { what: Selector, power: Value, toughness: Value, duration: Duration },
    /// CR 613.7d — switch each resolved permanent's power and toughness for
    /// `duration` (layer-7d, applied after all other P/T changes). Twisted
    /// Image, Wandering Fumarole's animated `{0}` ability.
    SwitchPT { what: Selector, duration: Duration },
    /// CR 613.4b — base P/T are `power` / `toughness` for `duration`, re-read
    /// every layer pass with the permanent's controller as "you" (Svogthos).
    SetBasePtLive { what: Selector, power: Value, toughness: Value, duration: Duration },
    /// Animate each `what` permanent into a creature for `duration` (manlands —
    /// Mutavault): layer-4 `AddCardType(Creature)` and subtypes, layer-7b
    /// `SetPowerToughness`, layer-6 keywords. It keeps its other types.
    BecomeCreature {
        what: Selector,
        power: Value,
        toughness: Value,
        creature_types: Vec<crate::card::CreatureType>,
        keywords: Vec<Keyword>,
        duration: Duration,
    },
    /// CR 205.1b — "becomes a P/T [types] creature", replacing its other card
    /// types (Opal Guardian). Indefinite; `SetCardTypesTo` reverts it by a later
    /// layer-4 timestamp.
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
    /// `SetCardTypesTo` for `duration` — "until your next turn, he becomes a
    /// land … (He's not a creature during that time.)" (Hydro-Man, Fluid
    /// Felon). The layer-4 replacement ends with the duration.
    SetCardTypesFor { what: Selector, card_types: Vec<crate::card::CardType>, duration: Duration },
    /// "Becomes an artifact creature" for `duration`: adds `CardType::Creature`
    /// (layer 4) only, keeping printed P/T — right for Vehicles (Guidelight
    /// Matrix). Unlike `BecomeCreature`, never sets P/T.
    AnimateAsCreature { what: Selector, duration: Duration },
    /// CR 611.2c — `keyword` to every `filter` permanent for the turn, not locked
    /// in at resolution, so later arrivals get it too (Falter). `filter` is
    /// matched card-locally: keywords from other effects aren't seen.
    GrantKeywordToMatchingThisTurn { filter: SelectionRequirement, keyword: Keyword },
    /// CR 611.2c / 613.1f — each `what` (fixed at resolution) loses `keyword` and
    /// can't regain it this turn (Arcane Lighthouse rulings): a layer-6
    /// `CantHaveKeyword`.
    CantHaveKeywordThisTurn { what: Selector, keyword: Keyword },
    /// CR 613 layer 7b — "[what]'s base power becomes `power`" for `duration`,
    /// leaving base toughness intact (Belligerent Yearling: base power becomes
    /// equal to the entering Dinosaur's power until end of turn).
    SetBasePower { what: Selector, power: Value, duration: Duration },
    GrantKeyword { what: Selector, keyword: Keyword, duration: Duration },
    /// `what` has `keyword` through its controller's next combat phase —
    /// "that creature attacks during its controller's next combat phase if
    /// able" with `MustAttack` (Sizzling Soloist).
    GrantKeywordThroughControllersNextCombat { what: Selector, keyword: Keyword },
    /// Several keywords on one `what` (one target slot) — Case of the Shattered
    /// Pact; a `Seq` of `GrantKeyword`s would declare a target each.
    GrantKeywords { what: Selector, keywords: Vec<Keyword>, duration: Duration },
    /// `what` loses `keyword` for `duration` (CR 613.7 layer 6, outranking
    /// earlier grants): Shadowspear; `Duration::Permanent` for Ageless
    /// Sentinels.
    LoseKeyword { what: Selector, keyword: Keyword, duration: Duration },
    /// CR 614 — Blind Fury: this turn, combat damage creature-to-creature is
    /// doubled; player and noncombat damage untouched.
    DoubleCombatDamageToCreaturesThisTurn,
    /// CR 701.33 — abandon `this` scheme: turn it face down and put it on the
    /// bottom of its owner's scheme deck. The ongoing-scheme escape hatch
    /// ("abandon this scheme" — Perhaps You've Met My Cohort).
    AbandonThisScheme,
    /// CR 702.15 — "Target creature loses all landwalk abilities" (Hammerheim).
    /// Layer-6 removal of every landwalk variant at once, which
    /// `LoseKeyword`'s exact match can't express.
    LoseAllLandwalk { what: Selector, duration: Duration },
    /// "… lose hexproof, protection, … and ward" (Shay Cormac): every
    /// hexproof, protection and ward instance each permanent has as this
    /// resolves, any variant, as `LoseAllLandwalk` does landwalk.
    LoseProtectiveKeywords { what: Selector, duration: Duration },
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
    /// CR 728 — restart the game: no winner, a fresh game from every card, the
    /// controller starting (727.1a). Cards exiled with the source are exempt
    /// (727.5) and with `deploy_source_exiles` enter under the controller (Karn
    /// Liberated's −14).
    RestartGame { deploy_source_exiles: bool },
    /// CR 305.1 — "Target player can't play lands this turn" (Turf Wound).
    /// Sets `Player.cant_play_lands_this_turn`, cleared at the turn boundary.
    PlayerCantPlayLandsThisTurn { player: PlayerRef },
    /// "Target player can't cast [filter] spells this turn" (Cease-Fire).
    /// Checked at the cast gate; cleared at cleanup.
    PlayerCantCastMatchingThisTurn { who: PlayerRef, filter: SelectionRequirement },
    /// "Each opponent can't cast [filter] spells during that player's next
    /// turn" (Azor, the Lawbringer): each player `who` names gets a
    /// `PlayerCantCastMatchingThisTurn` lock as their next turn begins.
    PlayerCantCastMatchingDuringTheirNextTurn { who: PlayerRef, filter: SelectionRequirement },
    /// "Until your next turn, [who] can't play cards from their hand" (Memory
    /// Vessel): arms `Player.hand_locked_until_turn_of` with the resolving
    /// controller's seat; it clears as that seat's next turn begins.
    PlayersCantPlayFromHandUntilYourNextTurn { who: PlayerRef },
    /// CR 602 — "that player can't activate abilities that aren't mana
    /// abilities" this turn (Abeyance). Sets
    /// `Player.cant_activate_nonmana_abilities_this_turn`, cleared at cleanup.
    PlayerCantActivateNonManaAbilitiesThisTurn { who: PlayerRef },
    /// "Look at `who`'s hand and choose `count` cards from it. Put them on top
    /// of that player's library in any order" (Agonizing Memories). The
    /// resolving controller picks; the auto-picker takes the priciest cards.
    ChooseFromHandToTopOfLibrary { who: PlayerRef, count: Value },
    /// "Until end of turn, you may cast instant and sorcery spells from the top
    /// of your graveyard. If a spell cast this way would be put into a
    /// graveyard, exile it instead" (Bösium Strip).
    CastFromGraveyardTopThisTurn,
    /// "You may cast creature spells this turn as though they had flash"
    /// (Winding Canyons). Sets `Player.creature_spells_as_flash_this_turn`.
    GrantCreatureSpellsFlashThisTurn { who: PlayerRef },
    /// As `GrantCreatureSpellsFlashThisTurn`, "until your next turn" (Arlinn).
    GrantCreatureSpellsFlashUntilYourNextTurn { who: PlayerRef },
    /// "You may cast spells this turn as though they had flash"
    /// (Alchemist's Refuge). Sets `Player.spells_as_flash_this_turn`,
    /// cleared at the turn boundary.
    GrantSpellsFlashThisTurn { who: PlayerRef },
    /// "You may cast sorcery spells this turn as though they had flash"
    /// (Complete the Circuit). Sets `Player.sorcery_spells_as_flash_this_turn`.
    GrantSorcerySpellsFlashThisTurn,
    /// "Creatures you control can't be the targets of spells or abilities your
    /// opponents control this turn" (Veilstone Amulet). Sets
    /// `Player.creatures_untargetable_by_opponents_this_turn`.
    YourCreaturesUntargetableByOpponentsThisTurn,
    /// Parade! (Starlight Spectacular): the controller lines up every creature
    /// they control one at a time; each gets +1/+1 until end of turn for each
    /// creature chosen before it. Headless, the smallest go first.
    Parade,
    /// Slaughter the Strong: each player keeps creatures with total power ≤ `max`
    /// (greedily, greatest power first; 0-power always fits) and sacrifices the
    /// rest at once.
    EachPlayerKeepsTotalPowerAtMost { max: u32 },
    /// Tree of Redemption — "Exchange your life total with this creature's
    /// toughness": your life becomes its toughness and its base toughness
    /// becomes your former life total (a layer-7b set, CR 613.4b).
    ExchangeLifeWithSourceToughness,
    /// Evra, Halcyon Witness — "Exchange your life total with this creature's
    /// power": your life becomes its power and its base power becomes your
    /// former life total (layer 7b, CR 613.4b).
    ExchangeLifeWithSourcePower,
    /// Tree of Perdition — "Exchange target opponent's life total with this
    /// creature's toughness": `who`'s life becomes its toughness and its base
    /// toughness becomes their former life total (CR 119.7, 613.4b).
    ExchangePlayerLifeWithSourceToughness { who: PlayerRef },
    /// "Until the end of [who]'s next turn, that player gets `amount` rad
    /// counters whenever they cast a spell" (Nuka-Nuke Launcher). Stacks
    /// per grant; cleared at the turn boundary after their next turn.
    RadOnCastUntilEndOfTheirNextTurn { who: PlayerRef, amount: u32 },
    /// "You and [opponent] each secretly choose 1 through `max`; if they match,
    /// [on_match], else [on_miss]" (Expert-Level Safe). A prompting seat names its
    /// number (`ask_seat_option`), any other picks uniformly — the equilibrium.
    /// `on_miss` reads the opponent's guess as `Value::TriggerEventAmount` (The
    /// Toymaker's Trap). `fresh`: only numbers the source hasn't chosen
    /// (`CardCold::chosen_numbers`); none left, nothing happens.
    SecretNumbersMatch {
        opponent: PlayerRef,
        max: u32,
        on_match: Box<Effect>,
        on_miss: Box<Effect>,
        #[serde(default)]
        fresh: bool,
    },
    /// Itazura, Lingering Wick — each opponent secretly chooses a number 0 or
    /// greater; the source deals the highest to an opponent who chose it (the
    /// controller breaks a tie), who may cast a spell from among `what` free;
    /// the controller puts one card of `what` that wasn't cast into hand.
    OpponentsBidDamageForFreeCast { what: Selector },
    /// Miss Highwater — `who` gets a contract counter for `beneficiary`: while
    /// they have it, when they lose the game the beneficiary creates a token
    /// copy of each artifact and creature they controlled.
    GiveContractCounter { who: PlayerRef, beneficiary: PlayerRef },
    /// Volo's Journal: note one of the first `what` spell's unnoted creature
    /// types (changelings offer the suggested types) in the source's
    /// `CardCold::noted_creature_types`.
    NoteCreatureTypeOf { what: Selector },
    /// Overencumbered's payment half: the enchanted player may pay {1} per
    /// artifact, else creatures can't attack this combat.
    EnchantedPlayerPaysPerArtifactOrNoAttacks,
    /// Vault 13: Dweller's Journey: return `count` cards exiled with this, bottom
    /// the rest. Headless returns its own first, then the priciest.
    ReturnSomeExiledWithSourceRestToBottom { count: u32 },
    /// "Destroy any number of target [filter] with total mana value `cap` or
    /// less" (Rampaging Yao Guai), picked on resolution: the controller chooses;
    /// a headless seat takes the opponents' priciest that fit.
    DestroyWithinTotalManaValue { filter: SelectionRequirement, cap: Value },
    /// Harold and Bob, First Numens: return the source transformed to its back
    /// face attached to the first `host` (an Aura back face enters as one).
    ReturnSelfTransformedAttachedTo { host: Selector },
    /// CR 303.4f — each `what` graveyard card enters under you enchanting the
    /// first `host` player (Lynde, Cheerful Tormentor's Curse).
    ReturnToBattlefieldAttachedToPlayer { what: Selector, host: PlayerRef },
    /// CR 303.4f / 701.23 — search for a `filter` card and put it onto the
    /// battlefield attached to player `host`, then shuffle (Bitterheart Witch;
    /// picks the priciest). `distinct_names`: no name shared with an Aura already
    /// on that player (Curse of Misfortunes).
    SearchAttachToPlayer { filter: crate::card::SelectionRequirement, host: PlayerRef, distinct_names: bool },
    /// CR 701.3 — move `filter` Auras on `from` to opponents (one each,
    /// controller's choice; headless: highest life). `all` moves every one, else
    /// one; `then` runs only if one moved (Lynde, the Unluckiest Planeswalker).
    ReattachPlayerAurasToOpponents {
        from: PlayerRef,
        filter: crate::card::SelectionRequirement,
        all: bool,
        then: Option<Box<Effect>>,
    },
    /// CR 707.10c — each player other than `except` may copy the spell
    /// `what` names, choosing new targets for the copy they control (Curse of
    /// Echoes). Each seat is asked; a headless seat copies.
    EachOtherPlayerMayCopy { what: Selector, except: PlayerRef },
    /// CR 509.1 / 506.4 — General Jarkeld: if each of two blocked attackers could
    /// be blocked by the other's blockers, every creature blocking exactly one
    /// switches to the other.
    SwitchBlockers { first: Selector, second: Selector },
    /// CR 701.52a — "roll to visit your Attractions": roll a d6 (a die roll,
    /// even with no Attraction out), then visit each Attraction you control
    /// with that number lit up (Line Cutter).
    RollToVisitAttractions,
    /// CR 702.159b — claim the source Attraction's prize, emitting the claim
    /// event (The Most Dangerous Gamer); the prize itself is the following
    /// effects.
    ClaimThePrize,
    /// CR 706.6 — Bamboozling Beeble: `who`'s next roll this turn adds a die and
    /// you ignore one (the lowest of a friend's, the highest of an opponent's).
    NextRollExtraDieThisTurn { who: PlayerRef },
    /// CR 701.30 — "[body], then clash with an opponent. If you win, repeat
    /// this process" (Hoarder's Greed). Each win re-enters this effect, so a
    /// suspended clash resumes without re-running an earlier `body`.
    RepeatWhileClashWon { body: Box<Effect> },
    /// "[body]. If [again], you may repeat this process" with no bound
    /// (Delina, Wild Mage's "15-20: … and you may roll again"). Each yes
    /// re-enters this effect, like `RepeatWhileClashWon`.
    RepeatWhileMay { body: Box<Effect>, again: Predicate, description: String },
    /// Eventide's Shadow: strip all counters from chosen permanents; draw and
    /// lose life per counter.
    RemoveCountersFromAmongDrawAndLoseLife,
    /// Audacious Swap: the owner shuffles the target in and exiles their top
    /// card; a land enters, otherwise they may cast it free (else it stays
    /// exiled).
    OwnerShufflesInExilesTopPlaysOrCasts { what: Selector },
    /// "Double the value of X" for the spell `what` resolves to while it is
    /// on the stack (Unbound Flourishing's cast trigger) — the stack item's
    /// X doubles, so an "enters with X counters" reads the doubled value.
    DoubleXOfSpell { what: Selector },
    /// CR 705 — Desperate Gambit: choose your source and flip; win: its next
    /// damage this turn doubles, lose: it is prevented.
    CoinFlipDoubleOrPreventNextDamage { filter: SelectionRequirement },
    /// "Search your library and graveyard for five cards and exile the rest.
    /// Put the chosen cards on top of your library in any order. You lose half
    /// your life, rounded up." (Doomsday.)
    Doomsday,
    /// "Tap all lands that player controls that could produce any type of mana
    /// the triggering land could produce" (Mana Web). `land` resolves the land
    /// that was tapped for mana.
    TapLandsSharingProductionWith { land: Selector },
    /// Tariff: each player sacrifices their priciest creature unless they pay
    /// its mana cost; ties broken by its controller (auto: cheapest to keep).
    EachPlayerSacrificesGreatestManaValueUnlessPays,
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
    /// "Becomes the color or colors of your choice" (Scuttlemutt, Quickchange):
    /// `BecomeChosenColor` whose choice is any nonempty set of colors. A
    /// headless seat still names its one best color.
    BecomeChosenColors { what: Selector, duration: Duration },
    /// CR 205.1b — the creature-type sibling of `BecomeChosenColor`: one named
    /// type for every `what` permanent for `duration` (Unnatural Selection).
    /// `excluded` is the printed restriction ("other than Wall" — Standardize);
    /// the decision doesn't offer it and overrules a decider that names it.
    BecomeChosenCreatureType {
        what: Selector,
        duration: Duration,
        #[serde(default)]
        excluded: Vec<crate::card::CreatureType>,
    },
    /// Each permanent picked by `what` becomes legendary for `duration`
    /// (CR 205.4 / layer-4 `AddSupertype`) — Cacophony Unleashed's "becomes
    /// a legendary 6/6 … until end of turn".
    BecomeLegendary { what: Selector, duration: Duration },
    /// Each permanent picked by `what` gains (`gain`) or loses `supertype`
    /// for `duration` (CR 205.4, layer 4) — Thermal Flux's "becomes snow" /
    /// "isn't snow until end of turn".
    ChangeSupertype { what: Selector, supertype: crate::card::Supertype, gain: bool, duration: Duration },
    /// `what` becomes exactly `colors` for `duration` (CR 105, layer-5
    /// `SetColors`) — fixed-color `BecomeChosenColor` (Crimson Wisps).
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
    /// CR 613 layer 7d — "switch target creature's power and toughness" for
    /// `duration` (Transmutation).
    SwitchPowerToughness { what: Selector, duration: Duration },
    /// Each `what` permanent's creature types become exactly `creature_types` for
    /// `duration` (CR 305.7, layer-4 `SetCreatureTypes`) — the type-line half of
    /// "becomes a [color] [type]", paired with `BecomeColor` / `SetBasePT` /
    /// `LoseAllAbilities` (Turn to Frog).
    BecomeCreatureType { what: Selector, creature_types: Vec<crate::card::CreatureType>, duration: Duration },
    /// CR 205.1b / 613.4 — `creature_types` added to each `what` permanent's own
    /// for `duration` (layer-4 `AddCreatureType`), unlike `BecomeCreatureType`
    /// (Jenova, Ancient Calamity).
    AddCreatureTypes { what: Selector, creature_types: Vec<crate::card::CreatureType>, duration: Duration },
    /// CR 205.3 — "gains all creature types" for `duration`: a layer-4
    /// `Modification::EveryCreatureType`, ordered against type-setting
    /// effects by timestamp (Amoeboid Changeling).
    GainAllCreatureTypes { what: Selector, duration: Duration },
    /// CR 612 — change the target's text by replacing all instances of one
    /// color word with another, both chosen by the controller (layer 3;
    /// rewrites Protection-from-color). Trait Doctoring, Mind Bend.
    ReplaceColorWord { what: Selector, duration: Duration },
    /// CR 612 / 305.7 — replace one basic land type with another in the target's
    /// text, both chosen (layer 3; type line and landwalk). Mind Bend.
    ReplaceBasicLandType { what: Selector, duration: Duration },
    /// CR 612.1 — replace every instance of one creature type with another in a
    /// spell or permanent's text, indefinitely; never Wall (CR 205.3m)
    /// (Artificial Evolution). It rewrites the *definition*, not a layer-3
    /// effect, so it survives entering the battlefield and reaches ability text.
    /// `to` set: only the replaced type is chosen (New Blood).
    ReplaceCreatureTypeText {
        what: Selector,
        #[serde(default)]
        to: Option<crate::card::CreatureType>,
    },
    /// The controller chooses a color as the source enters; stamp it onto the
    /// source's `chosen_color` (CR 614 — Coldsteel Heart, choose-a-color mana
    /// rocks). Read later by `ManaPayload::ChosenColorOfSource`.
    ChooseColorForSelf,
    /// `ChooseColorForSelf` with one color ruled out — the Thriving lands'
    /// "choose a color other than green".
    ChooseColorForSelfOtherThan(crate::mana::Color),
    /// `ChooseColorForSelf` from a printed short list — Mangara's Equity's
    /// "choose black or red".
    ChooseColorForSelfAmong(Vec<crate::mana::Color>),
    /// CR 205.1b — `what` stops being a `creature_type` for `duration`
    /// ("and isn't a Human" — Werewolf Pack Leader); its other types stay.
    LoseCreatureType { what: Selector, creature_type: crate::card::CreatureType, duration: Duration },
    /// Tablet of the Guilds: as this enters, choose two colors (stamped on
    /// `CardInstance.chosen_colors`).
    ChooseTwoColorsForSource,
    /// Tablet of the Guilds: whenever you cast a spell that is at least one of
    /// the source's chosen colors, gain 1 life for each chosen color it is.
    GainLifePerChosenColorOfCast,
    /// Protection from a chosen color for `duration` (`Decision::ChooseColor` →
    /// `Keyword::Protection`). Mother of Runes.
    GrantProtectionFromChosenColor { what: Selector, duration: Duration },
    /// [`Effect::GrantProtectionFromChosenColor`] with colorless as one more
    /// choice (Giver of Runes): taken when the threat on the stack is
    /// colorless.
    GrantProtectionFromChosenColorOrColorless { what: Selector, duration: Duration },
    /// "… gains protection from the card type of your choice" (Pippin, Guard
    /// of the Citadel): the controller names a card type as this resolves.
    GrantProtectionFromChosenCardType { what: Selector, duration: Duration },
    /// Skrelv, Defector Mite: choose a color; hexproof from it and unblockable by
    /// it this turn. Chosen as for `GrantProtectionFromChosenColor` (the
    /// threat's, else the densest among opponents).
    GrantHexproofAndEvasionFromChosenColor { what: Selector, duration: Duration },
    /// Mondo Gecko: becomes a chosen color (layer 5, picked as for
    /// `GrantHexproofAndEvasionFromChosenColor`) with `HexproofFromColor` of it.
    BecomeChosenColorWithHexproof { what: Selector, duration: Duration },
    /// CR 702.16 — protection from player `from` for `duration` (`EndOfTurn` or
    /// `UntilNextTurn`): permanents get `ProtectionFromMatching(ControlledBySeat)`;
    /// a player can't be targeted or damaged by what that player controls
    /// (Guardian Archon).
    GainProtectionFromPlayer { what: Selector, from: PlayerRef, duration: Duration },
    /// CR 509.1b — "can't be blocked by creatures [by] controls" for `duration`
    /// (`CantBeBlockedBy(ControlledBySeat ∨ …)`). The Black Gate.
    CantBeBlockedByPlayer { what: Selector, by: PlayerRef, duration: Duration },
    /// CR 603.7d — until your next turn, a delayed trigger per `what` permanent
    /// firing `body` on its combat damage (Tamiyo, Field Researcher).
    /// `to_player`: player damage only.
    WatchCombatDamageUntilYourNextTurn {
        what: Selector,
        body: Box<Effect>,
        #[serde(default)]
        to_player: bool,
    },
    /// A transient triggered ability on each `what` permanent for `duration`, in
    /// `GameState.granted_triggers_eot` (`Duration::Permanent` bakes it into the
    /// definition — Volver kickers). The dispatcher reads printed and granted
    /// triggers alike (Root Manipulation). The dies half of Rabid Attack needs an
    /// LTB-snapshot follow-up.
    GrantTriggeredAbility {
        what: Selector,
        trigger: Box<crate::card::TriggeredAbility>,
        duration: Duration,
    },
    /// CR 603.8 / 113.10 — each permanent `what` picks gains the state
    /// trigger, baked onto it until it leaves (Olivia, Crimson Bride's
    /// "When you don't control a legendary Vampire, exile this creature").
    GrantStateTrigger { what: Selector, trigger: Box<crate::card::StateTriggeredAbility> },
    /// "Loses all abilities": a layer-6 `Modification::RemoveAllAbilities`, which
    /// clears keywords and sets `lost_all_abilities` so triggers and activations
    /// skip printed abilities (CR 113.10b). Turn to Frog, Lignify.
    LoseAllAbilities { what: Selector, duration: Duration },
    AddCounter    { what: Selector, kind: CounterType, amount: Value },
    /// "Put up to `amount` `kind` counters on [what]. This ability can't cause
    /// the total number of `kind` counters on it to be greater than `cap`."
    /// Clamps per target against its current pool (Clockwork Avian).
    AddCounterCapped { what: Selector, kind: CounterType, amount: Value, cap: Value },
    /// "Put up to `max` `kind` counters on [what]": one count for every resolved
    /// permanent (Esper Terra); `filter` narrows them.
    AddCountersUpTo {
        what: Selector,
        kind: CounterType,
        max: Value,
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },
    RemoveCounter { what: Selector, kind: CounterType, amount: Value },
    /// CR 603.7e — until the end of your next turn, `body` on each spell you
    /// cast (Season of the Bold); the spell is `TriggerSource`.
    OnEachSpellYouCastUntilEndOfYourNextTurn { body: Box<Effect> },
    /// Call the Spirit Dragons: per color (WUBRG) put a `kind` counter on a
    /// `filter` permanent of that color; win with `win_at` distinct recipients
    /// (a multicolored one can be picked twice).
    CounterOnMatchingOfEachColor {
        filter: SelectionRequirement,
        kind: CounterType,
        win_at: u32,
    },
    /// CR 603.7e — "when you next cast a creature spell this turn, it enters with
    /// N `kind` counters": a one-shot rider on
    /// `Player.pending_creature_etb_counters`, expiring at cleanup (FIN Summon
    /// sagas).
    GrantNextCreatureSpellCounters { kind: CounterType, amount: Value },
    /// CR 603.7e — your next creature spell this turn gains `keyword` as it
    /// enters (`Player.pending_creature_etb_keywords`; Summon: Brynhildr).
    GrantNextCreatureSpellKeyword { keyword: Keyword },
    /// For each permanent matching `filter` whose current power exceeds its
    /// base power, put that many +1/+1 counters on it (the per-permanent
    /// difference). Sovereign Okinec Ahau's attack trigger (CR 122).
    AddCountersForPowerOverBase { filter: SelectionRequirement },
    /// CR 701.63 — endure N: `target`'s controller adds N +1/+1 counters or makes
    /// an N/N Spirit (AutoDecider: counters). N = 0 does nothing.
    Endure { target: Selector, n: Value },
    /// CR 701.68 — *blight N*: the controller puts N -1/-1 counters on a
    /// creature they control (their choice). With no creature they can't
    /// blight (701.68b) so it's a no-op. N=0 does nothing.
    Blight { n: Value },
    /// "Blight N. If you do, [then]": `then` runs only when a creature took the
    /// counters, reading it as `Selector::LastMoved` (Grub, Notorious
    /// Auntie's attacking copy of the blighted creature).
    BlightThen { n: Value, then: Box<Effect> },
    /// CR 701.66 — *earthbend N*: target land you control becomes a 0/0 land
    /// creature with haste (in addition to its other types) and gets N +1/+1
    /// counters; a delayed trigger returns it tapped when it dies or is exiled.
    Earthbend { n: Value },
    /// CR 701.65 — airbend: exile each `what`; its owner may cast it for {2}
    /// while exiled. Wrap in `ApplyToTargets` for "up to one target".
    Airbend { what: Selector },
    /// Remove every counter of every kind from `what` (CR 122.6 — Vampire
    /// Hexmage's "remove all counters from target permanent"). The count
    /// feeds `Value::CountersRemovedThisEffect` (Thief of Blood).
    RemoveAllCounters { what: Selector },
    /// "Remove all counters from any number of [filter]" (Sin, Unending
    /// Cataclysm) — the controller's pick among matching permanents with
    /// counters; the total feeds `Value::CountersRemovedThisEffect`.
    RemoveAllCountersFromAnyNumber { filter: SelectionRequirement },
    /// Remove a single counter of any one kind from `what` (the controller's
    /// choice; auto-picks the first present kind). Thrull Parasite's "remove a
    /// counter from target nonland permanent".
    RemoveAnyCounter { what: Selector },
    /// Remove up to `amount` counters (any kinds, controller's choice) from
    /// `what`; a player loses poison (CR 122.6 — Price of Betrayal). The pick
    /// first takes kinds whose removal helps the remover (harmful off its own
    /// side, any off an opponent's); `only_helpful` stops there (Heartless Act).
    RemoveCountersUpTo {
        what: Selector,
        amount: Value,
        #[serde(default)]
        only_helpful: bool,
    },
    /// Sensational Spider-Man: remove up to `max` `kind` counters from `filter`
    /// permanents (headless: your own first); count on
    /// `Value::CountersRemovedThisEffect`.
    RemoveCountersOfKindAmongUpTo { kind: crate::card::CounterType, max: Value, filter: SelectionRequirement },
    /// CR 606 — set `what`'s loyalty to `value`, overwriting the counter count
    /// (Geyadrone Dihada).
    SetLoyalty { what: Selector, value: Value },
    /// "You may activate loyalty abilities of ~ twice this turn rather than
    /// only once" — Kaito, Dancing Shadow's combat-damage rider. Sets the
    /// resolved planeswalkers' `loyalty_twice_this_turn` (cleared at cleanup).
    GrantLoyaltyTwiceThisTurn { what: Selector },
    /// CR 702.65 — aura swap: this Aura returns to hand and a hand Aura enters on
    /// the same permanent (Arcanum Wings). Auto: priciest; UI via ChooseCards.
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
    /// "Put those counters on [to]" (Resourceful Defense): every counter of
    /// every kind on `from` — read through last-known information when it has
    /// left the battlefield (CR 603.10a) — is put on each `to`.
    PutCountersOf { from: Selector, to: Selector },
    /// CR 702.44 — "[the spell] gains sunburst" (Lux Artillery, Solar Array):
    /// each spell `what` resolves to enters with a +1/+1 counter (creature) or
    /// charge counter (otherwise) per color of mana spent to cast it.
    SpellGainsSunburst { what: Selector },
    /// Rakdos, the Showstopper — "flip a coin for each creature that isn't one
    /// of `exclude_types`; destroy each creature whose coin comes up tails."
    /// The controller flips (honoring coin-flip advantage).
    CoinFlipEachCreatureDestroyOnTails { exclude_types: Vec<crate::card::CreatureType> },
    /// Awaken the Erstwhile — "each player discards all the cards in their
    /// hand, then creates that many `token` tokens." Resolved in turn order.
    EachPlayerDiscardsHandMakeTokens { token: std::sync::Arc<crate::card::TokenDefinition> },
    /// Memory Jar: each player exiles their hand face down and draws seven; at
    /// the next end step they discard and take back the exiles. One effect with
    /// its own delayed trigger.
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
    /// Galloping Lizrog: remove +1/+1 counters from your creatures and put twice
    /// that many on this. "Any number" is collapsed to all other creatures;
    /// counter doublers apply (CR 614.16).
    DoubleP1P1CountersFromYourCreatures,
    /// CR 702.43c — Modular: move the dying source's last-known +1/+1 counters to
    /// the target. `StaticEffect::ModularBonusCounters` (Zabaz) adds its bonus.
    ModularCounters { what: Selector },
    /// CR 702.62a-f — exile with `time_counters`, gaining suspend if needed (the
    /// card Suspend); `process_suspend` ticks and free-casts it.
    GrantSuspend { what: Selector, time_counters: u32 },
    /// "Choose a counter on target permanent or suspended card. Remove that
    /// counter or put another of those counters on it" (Clockspinning). The
    /// bot helps its own objects and hurts an opponent's.
    Clockspin { what: Selector },
    /// CR 702.26 — every permanent phased out "until" this source phases in
    /// (The Pandorica, becoming untapped).
    PhaseInHeldBySource,
    /// Narrow the may-play permission on each selected exiled card to
    /// casting: "you may cast spells from among them" (Dream Pillager). Runs
    /// after the effect that granted it.
    RestrictMayPlayToCasting { what: Selector },
    /// "[This] has all activated and triggered abilities of the exiled card"
    /// (Idris, Soul of the TARDIS): the cards exiled with the source lend
    /// their abilities to it, stamped as they're exiled.
    AcquireAbilitiesOfExiledWithSource,
    /// Koh, the Face Stealer: choose a `filter` card exiled with the source
    /// (headless: the one lending most abilities); the source's borrowed
    /// abilities become that card's.
    BorrowAbilitiesOfChosenExiled { filter: crate::card::SelectionRequirement },
    /// "Choose up to `max` [keep]. Exile all other creatures" (The Day of
    /// the Doctor's IV, inside its "you may"). The controller chooses among
    /// every match, anyone's; headless, its own greatest-power ones.
    ExileOtherCreaturesKeepingUpTo { keep: crate::card::SelectionRequirement, max: u32 },
    /// CR 707.9b — "except it's a [type]" *instead of* its other creature types
    /// (Hour of Eternity; unlike eternalize).
    SetCopiableCreatureTypes { what: Selector, creature_types: Vec<crate::card::CreatureType> },
    /// CR 707.9b — a copy "except it's [types] named [name]" (The Eleventh Hour's
    /// Prisoner Zero): name and creature types join the copiable values.
    SetCopiableNameAndTypes {
        what: Selector,
        #[serde(with = "crate::static_str_serde")]
        name: crate::static_str_serde::StaticStr,
        creature_types: Vec<crate::card::CreatureType>,
    },
    /// "Becomes … named [name]" — not a copy effect, so copies keep the printed
    /// name (CR 707.2). Baked via `CardInstance::bake_grant` until it leaves (The
    /// Curse of Fenric).
    BecomeNamed {
        what: Selector,
        #[serde(with = "crate::static_str_serde")]
        name: crate::static_str_serde::StaticStr,
    },
    /// "You may cast spells from your hand this turn without paying their
    /// mana costs" (Yusri's five-win jackpot). Sets the controller's
    /// `free_spells_from_hand_this_turn` flag, cleared at end-of-turn.
    FreeSpellsFromHandThisTurn,
    /// "As this enters, choose a card type" (Serra's Emissary). Asks the
    /// controller via `ChooseMode` over the permanent+spell card types and
    /// stamps `CardInstance.chosen_card_type` on the source.
    ChooseCardTypeForSource,
    /// `ChooseCardTypeForSource` over a printed list, in printed order (Cloud
    /// Key's "artifact, creature, enchantment, instant, or sorcery").
    ChooseCardTypeFromForSource { options: Vec<crate::card::CardType> },
    /// `ChooseCardTypeForSource` over the card types shared by two or more
    /// cards exiled with the source — "a card type shared among two exiled
    /// cards used to craft it" (Apex Observatory, CR 702.167c).
    ChooseCardTypeSharedByExiledWithSource,
    /// Apex Observatory: your next spell of the source's `chosen_card_type` this
    /// turn is free.
    NextSpellOfSourceChosenTypeFreeThisTurn,
    /// CR 123.2 — "You get {TK}{TK}": `amount` ticket counters on `who`
    /// (Command Performance). `Player.tickets`; nothing spends them yet (name
    /// stickers are free).
    GainTickets { who: PlayerRef, amount: u32 },
    /// Lonis's steal: target opponent reveals the top `count`; you may put a
    /// nonland permanent with mana value ≤ `max_mv` onto the battlefield (auto:
    /// the priciest), the rest bottom at random.
    OpponentRevealsPickToBattlefield { count: Value, max_mv: Value },
    /// Gaea's Will: this turn you may play lands and cast spells from your
    /// graveyard, later arrivals included (`CastFromGraveyardsThisTurn`'s
    /// registry; lands via the graveyard land-play gate).
    PlayFromGraveyardThisTurn,
    /// CR 701.46a — Adapt N: "if this permanent has no +1/+1 counters on it,
    /// put N +1/+1 counters on it" — or, once, as though it had none when an
    /// `AdaptAsThoughNoCounters` waiver names it this turn.
    Adapt { amount: Value },
    /// "The next time target creature adapts this turn, it adapts as though
    /// it had no +1/+1 counters on it" (Biomancer's Familiar). One waiver per
    /// resolution; the next `Adapt` by that creature consumes it.
    AdaptAsThoughNoCounters { what: Selector },
    /// "Until end of turn, you may cast [filter] spells from your / any
    /// graveyard" (The Great Work III): a turn permission, so later arrivals
    /// count. Own cost; `exile_after` exiles instead of the graveyard.
    CastFromGraveyardsThisTurn { any_graveyard: bool, filter: SelectionRequirement, exile_after: bool },
    /// Gaea's Will — "If a card would be put into your graveyard from
    /// anywhere this turn, exile it instead" (turn-scoped Rest in Peace,
    /// own cards only).
    ExileYourGraveyardBoundThisTurn,
    /// Glimpse of Tomorrow: shuffle your permanents in, reveal that many;
    /// non-Auras enter, then Auras (auto-attached), the rest bottom at random.
    GlimpseOfTomorrow,
    /// Garth One-Eye: pick an unchosen name from `names` and get a copy you may
    /// cast — approximated as a real card in hand
    /// (`CardInstance.name_choices_used`).
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
    /// Yusri, Fortune's Flame: choose 1..=`max`, flip that many; `per_win` /
    /// `per_loss` per flip, then `all_won` (reading the number as
    /// `Value::XFromCost`) if it was `all_won_min`+ and every flip won.
    /// `stop_on_loss` stops at the first loss (Squee's Revenge).
    FlipCoinsChooseCount {
        max: u32,
        per_win: Box<Effect>,
        per_loss: Box<Effect>,
        all_won: Box<Effect>,
        all_won_min: u32,
        #[serde(default)]
        stop_on_loss: bool,
    },
    /// Vraska, Betrayal's Sting's −2: becomes a Treasure artifact with "{T},
    /// Sacrifice: add one mana of any color", losing other types and abilities
    /// (layers 4 and 6). `while_source`: only while the source remains (Kitesail
    /// Larcenist).
    BecomeTreasure {
        what: Selector,
        #[serde(default)]
        while_source: bool,
    },
    /// CR 122.1b — keyword counters on `what`: it has the keyword while one
    /// remains (a layer-6 grant in `compute_battlefield`).
    AddKeywordCounter { what: Selector, keyword: crate::card::Keyword, amount: Value },
    /// CR 122.1b — a counter of a random kind `what` lacks (from
    /// `keyword_options`, or +1/+1 with `plus_one_plus_one`). Crystalline Giant.
    /// Nothing if it has them all.
    AddRandomMissingCounter {
        what: Selector,
        keyword_options: Vec<crate::card::Keyword>,
        #[serde(default)]
        plus_one_plus_one: bool,
    },
    /// CR 122.1b — remove up to `amount` `keyword` counters from `what`; the last
    /// one takes the keyword with it. Counterpart to `AddKeywordCounter`.
    RemoveKeywordCounter { what: Selector, keyword: crate::card::Keyword, amount: Value },
    /// CR 122.5 — move `amount` `kind` counters from `from` to `to` (clamped);
    /// one `CounterRemoved` and one `CounterAdded` per target. Moving isn't
    /// creating, so doublers (CR 614.16) don't apply. Tester of the Tangential.
    MoveCounter   { from: Selector, to: Selector, kind: CounterType, amount: Value },
    /// Move every counter (all kinds) from `from` onto `to` (The Ozolith's
    /// begin-combat transfer). Relocation, not creation — no doublers.
    MoveAllCounters { from: Selector, to: Selector },
    /// CR 122.8 — "remove any number of counters from [what]" (Rhys, the
    /// Evermore): per permanent and kind (keyword counters too) the controller
    /// picks how many. Headless: every kind its owner would rather lose.
    RemoveAnyNumberOfCounters { what: Selector },
    /// CR 122.5 — "move any number of counters from [from] onto [to]"
    /// (Resourceful Defense): per source and kind the controller picks how many.
    /// `from` may be several permanents; `to` is never a source; `kind` limits
    /// it to one kind.
    MoveAnyNumberOfCounters {
        from: Selector,
        to: Selector,
        #[serde(default)]
        kind: Option<crate::card::CounterType>,
    },
    /// CR 603.7 reflexive "when you do": push `body` as a trigger of the same
    /// source and controller instead of resolving inline, so opponents get a
    /// response window. Targets are picked at push (CR 603.7d). Rubble Rouser.
    ReflexiveTrigger { body: Box<Effect> },
    /// "Put the same number and kind of counters on …" (Bold Plagiarist): the
    /// kind of the triggering `CounterAdded`, rewritten to `AddCounter` as the
    /// trigger is created (`for_trigger_event`); no-op elsewhere. `event_placer`:
    /// the event's placer puts them (CR 122.6).
    AddCounterOfTriggerKind {
        what: Selector,
        amount: Value,
        #[serde(default)]
        event_placer: bool,
    },
    /// CR 701.34a Proliferate. Auto-decider baseline: grow good counters (+1/+1,
    /// Loyalty, Charge, Page) on yours, bad ones (-1/-1, Stun) on theirs, others
    /// by default, and poison on poisoned opponents. No multi-select UI yet.
    Proliferate,
    /// "For each [filter], choose a counter on it; you may add one of that kind"
    /// (The Caves of Androzani). Chosen like proliferate, but it isn't one (CR
    /// 701.34): no player counters, no Proliferated event.
    AddOneOfAChosenCounterToEach { filter: crate::card::SelectionRequirement },
    /// Gain control of `what`. `to` names the new controller (`None` = the
    /// effect's controller — Threaten; `Some` = Wishclaw Talisman's "target
    /// opponent gains control"). `#[serde(default)]` keeps old snapshots loading.
    GainControl {
        what: Selector,
        #[serde(default)]
        to: Option<PlayerRef>,
        duration: Duration,
    },
    /// Gain control of the resolved permanents for as long as the effect's
    /// source remains on the battlefield (Sower of Temptation).
    GainControlWhileSourceRemains { what: Selector },
    /// CR 611.2c — "gain control of `what` for as long as you control this
    /// creature" (Master Thief): the steal also ends when the source changes
    /// controller, not only when it leaves. Swept with Vedalken Shackles.
    GainControlWhileYouControlSource { what: Selector },
    /// CR 611.2c — Vedalken Shackles: control `what` while the source stays
    /// tapped (unwound by the SBA sweep); while it holds one the source skips
    /// untapping.
    GainControlWhileSourceTapped { what: Selector },
    /// Both clauses — "for as long as you control this and it remains tapped"
    /// (Rubinia Soulsinger, Helm of Possession).
    GainControlWhileYouControlSourceTapped { what: Selector },
    /// CR 611.2c — "gain control of `what` for as long as it has a `kind`
    /// counter on it" (Shield Broker's shield counter). The SBA sweep hands
    /// it back once the last such counter is gone.
    GainControlWhileCounter { what: Selector, kind: crate::card::CounterType },
    /// CR 611.2c — "gain control of `what` for as long as it matches
    /// `filter`" (Rootwater Matriarch: "for as long as that creature is
    /// enchanted"). The SBA sweep hands it back once it stops matching.
    GainControlWhileMatches { what: Selector, filter: crate::card::SelectionRequirement },
    /// CR 611.2c — Garland, Royal Kidnapper: control `what` while `monarch`
    /// (fixed at resolution) is the monarch; the SBA sweep returns it.
    GainControlWhileMonarch { what: Selector, monarch: PlayerRef },
    /// CR 611.2c — "gain control of that permanent for as long as that Aura is
    /// attached to it" (Eriette, the Beguiler). The Aura is
    /// `ctx.trigger_source`; the stolen permanent is whatever it's attached to.
    GainControlWhileTriggerAuraAttached,
    /// The `This`-keyed sibling of `GainControlWhileTriggerAuraAttached`: the
    /// effect's own source is the Aura holding the steal (Dominating Licid,
    /// whose attach is an activated ability rather than an ETB trigger).
    GainControlWhileSourceAttached,
    /// CR 611.2c sibling — the resolved permanents gain `keyword` for as long
    /// as the effect's source stays tapped on the battlefield (Hisoka's Guard's
    /// shroud grant). Unwound by the same SBA sweep.
    GrantKeywordWhileSourceTapped {
        what: Selector,
        keyword: crate::card::Keyword,
    },
    /// CR 611.2c sibling — "for as long as [source] remains tapped, if a
    /// triggered ability of [what] triggers, that ability triggers an
    /// additional time" (Lwaxana Troi, Charming Guest).
    DoubleTriggersWhileSourceTapped { what: Selector },
    /// Mindblaze: pick a number > 0; the player reveals their library; exactly
    /// that many of the named card deals `damage` to them. They shuffle.
    RevealLibraryNamedCountPunish { who: Selector, damage: Value },
    /// Moonring Mirror's upkeep — the controller may exile their whole hand
    /// face down stamped `exiled_with = source`; if they do, every *other*
    /// card they own already stamped that way returns to their hand.
    ExileHandThenReclaimLinked,
    /// Reweave: sacrifice `what`; its controller reveals until a permanent card
    /// sharing a type, puts it onto the battlefield, shuffles.
    SacrificeThenRevealUntilSharedType { what: Selector },
    /// The exile twin of `SacrificeThenRevealUntilSharedType` (Chaotic
    /// Transformation): exile `what`; its controller reveals until a card sharing
    /// a type, puts it onto the battlefield, shuffles
    /// (`game/exile_reveal_shared.rs`).
    ExileThenRevealUntilSharedType { what: Selector },
    /// Struggle for Sanity: the opponent reveals their hand; they and you
    /// alternately exile one (them first); theirs return, yours are binned.
    AlternatingExileFromHand { who: Selector },
    /// "Double the amount of each type of unspent mana you have" (Doubling
    /// Cube) — every color and colorless pip in the controller's pool is
    /// duplicated, restrictions included.
    DoubleUnspentMana,
    /// CR 614.5 — "If any source you control would deal damage … this turn,
    /// it deals double that damage instead." Sets the controller's
    /// turn-scoped flag read by `scale_damage_to` (Quest for Pure Flame).
    DoubleYourSourcesDamageThisTurn,
    /// CR 614.1a — "If a red source you control would deal damage to a
    /// permanent or player this turn, it deals that much damage plus `amount`
    /// instead" (The Flame of Keld's chapter III). Stacks per resolution.
    YourRedSourcesDealExtraDamageThisTurn { amount: u32 },
    /// CR 614.1a — "If a source would deal damage to a player or battle this
    /// turn, it deals that much damage plus `amount` instead" — every source,
    /// every player (Rankle and Torbran). Stacks per resolution.
    DamageToPlayersAndBattlesPlusThisTurn { amount: u32 },
    /// CR 614.1a — Isengard Unleashed: your sources' damage to opponents and
    /// their permanents is tripled this turn.
    TripleYourSourcesDamageToOpponentsThisTurn,
    /// CR 614.1a — Goblin Goliath: your sources' damage to opponent players (not
    /// permanents) is doubled this turn.
    DoubleYourSourcesDamageToOpponentPlayersThisTurn,
    /// CR 614.1a / 111.7 — Hallowed Moonlight: this turn, uncast creatures are
    /// exiled instead of entering; tokens are never made.
    ExileUncastCreaturesThisTurn,
    /// Goblin Welder: if both targets are still legal, artifact `a`'s
    /// controller simultaneously sacrifices it and returns artifact card `b`
    /// from their graveyard to the battlefield.
    WeldArtifacts { a: Selector, b: Selector },
    /// Create `count` copies of the given token under `who`'s control.
    CreateToken { who: PlayerRef, count: Value, definition: std::sync::Arc<TokenDefinition> },
    /// CR 614.13 — Kaya, Geist Hunter's −2: one more token doubling for `who`
    /// this turn, stacking with `StaticEffect::DoubleTokens`.
    DoubleTokensThisTurn { who: PlayerRef },
    /// Waiting in the Weeds: each player makes a `definition` token per
    /// `filter` they control — counted per player, unlike `CreateToken` with
    /// `EachPlayer`.
    EachPlayerCreatesTokenPerControlled {
        filter: SelectionRequirement,
        definition: std::sync::Arc<TokenDefinition>,
    },
    /// Tetravus: remove any number of `kind` counters, make that many tokens
    /// stamped `created_by = source` for
    /// `ExileTokensCreatedBySourceForCounters`.
    RemoveCountersToCreateTokens { kind: CounterType, definition: std::sync::Arc<TokenDefinition> },
    /// The mirror: "you may exile any number of tokens created with this. If
    /// you do, put that many `kind` counters on it" (Tetravus).
    ExileTokensCreatedBySourceForCounters { kind: CounterType },
    /// Exile every token minted this resolution at the next end step (after a
    /// `CreateToken` — Valduk, Keeper of the Flame).
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
    /// Amass N (CR 701.43): `count` +1/+1 counters on `who`'s Army, making a 0/0
    /// one first if needed; `extra_type` adds a subtype (Amass Orcs).
    Amass { who: PlayerRef, count: Value, extra_type: Option<crate::card::CreatureType> },
    /// Empower Jace N: `count` loyalty on your Jace token, first making one
    /// ("[−1]: Surveil 1", "[−3]: Draw a card") if needed — Amass on a walker.
    EmpowerJace { who: PlayerRef, count: Value },
    /// CR 606.3 override — Jace's Machinations: this turn, Jace loyalty abilities
    /// at instant speed on any turn (`game/jace_instant_loyalty.rs`).
    JaceLoyaltyAtInstantSpeedThisTurn,
    /// CR 606.3 override — Urza Assembles the Titans III: loyalty abilities twice
    /// this turn, later arrivals too (`game/loyalty_twice_turn.rs`).
    LoyaltyAbilitiesTwiceThisTurn,
    /// Create `count` tokens tapped and attacking (CR 508.3a) the source's
    /// defender, else the controller's first opponent — Mobilize (CR 702.181).
    /// `cleanup` removes them at end of combat. No-op outside combat.
    CreateTokenAttacking {
        who: PlayerRef,
        count: Value,
        definition: std::sync::Arc<TokenDefinition>,
        #[serde(default)]
        cleanup: AttackingTokenCleanup,
        /// Whom the tokens attack. `None`: the source's defender, else the first
        /// opponent. `Some`: "attacking that player" (Adeline under
        /// `ForEachOpponent`).
        #[serde(default)]
        defender: Option<PlayerRef>,
    },
    /// CR 508.3a — put the resolved permanents into combat tapped and attacking
    /// the source's defender (else the first opponent), skipping declare-attackers
    /// gates. After a tapped `Move` to the battlefield it reanimates "tapped and
    /// attacking" (Alesha, via `LastMoved`). No-op outside combat.
    JoinCombatAttacking { what: Selector },
    /// CR 508.4 — `JoinCombatAttacking` with no named defender: each one's
    /// controller picks a player or planeswalker (headless: the source's
    /// defender). Flamerush Rider. `cleanup` applies to every mover, doubler
    /// tokens included.
    JoinCombatAttackingChosen {
        what: Selector,
        #[serde(default)]
        cleanup: AttackingTokenCleanup,
    },
    /// Exile the top card of the source controller's library; if it's a
    /// creature card, the source gets +power/+toughness until end of turn equal
    /// to that card's power and toughness. Bioplasm's attack trigger.
    ExileTopSelfPumpIfCreature,
    /// Myriad (CR 702.116): per opponent other than the one attacked, a tapped
    /// and attacking token copy of the source, exiled at end of combat. No-op
    /// unless the source is attacking a player.
    Myriad,
    /// Shredder, Shadow Master: Myriad with its own terms — non-legendary copies,
    /// `cleanup` at end of combat, and the source must be attacking a player.
    CopiesAttackEachOtherOpponent { non_legendary: bool, cleanup: AttackingTokenCleanup },
    /// Gyrus, Waker of Corpses: a tapped and attacking token copy of `source`
    /// (found on the battlefield, in exile or a graveyard; CR 707.2), exiled at
    /// end of combat (`sacrifice`: sacrificed — Phantom Steed). It attacks the
    /// source's defender, else the first opponent, never declared (CR 508.4).
    /// No-op outside combat. The token is `LastCreatedToken`.
    TokenCopyAttackingUntilEndOfCombat {
        source: Selector,
        #[serde(default)]
        sacrifice: bool,
    },
    /// A tapped and attacking token copy of `source` with no cleanup, as
    /// `LastCreatedToken` for a chained rider (Satya, Aetherflux Genius). Attacks
    /// the source's defender, else the hostile opponent; no-op outside combat.
    TokenCopyTappedAttacking { source: Selector },
    /// "The next instant or sorcery you cast this turn costs {amount} less"
    /// (Thundertrap Trainer). `amount` is read at resolution; a one-shot
    /// discount on `Player.pending_is_discounts`.
    GrantNextInstantOrSorceryDiscountThisTurn { amount: Value },
    /// "The next spell you cast this turn costs {amount} less to cast" — any
    /// spell (Peerless Samurai). Pushes onto `Player.pending_spell_discounts`,
    /// keyed on the spells-cast tally so it lapses with the next spell.
    GrantNextSpellDiscountThisTurn { amount: Value },
    /// "The next [filter] spell you cast this turn costs {amount} less"
    /// (Invasion of the Giants): waits on `Player.next_matching_spell_discounts`
    /// for the first matching cast; others pass it by. Cleared at end of turn.
    GrantNextMatchingSpellDiscountThisTurn { filter: crate::card::SelectionRequirement, amount: u32 },
    /// "The next [filter] spell you cast this turn has undaunted" (Võx, Child
    /// of the Collective; CR 702.125): the discount grant above at {1} per
    /// opponent, counted as this resolves.
    GrantNextMatchingSpellUndauntedThisTurn { filter: crate::card::SelectionRequirement },
    /// "The next [filter] spell you cast this turn can be cast without paying
    /// its mana cost" (World War Hulk I): `Player.free_next_spells`, spent by
    /// the first matching cast, cleared at end of turn.
    GrantNextMatchingSpellFreeThisTurn { filter: crate::card::SelectionRequirement },
    /// Support N (CR 701.32): a +1/+1 counter on each of up to N optional
    /// `filter` creature targets.
    SupportCounters { max_targets: u8, filter: SelectionRequirement },
    /// CR 601.2d — distribute N `counter` counters among up to `max_targets`
    /// optional `filter` targets (Hatchery Spider), split at resolution via
    /// `Decision::DivideDamage` (AutoDecider spreads evenly).
    DistributeCounters { total: Value, counter: CounterType, filter: SelectionRequirement, max_targets: u8 },
    /// Untargeted `DistributeCounters` over `what` as it resolves (Stumpsquall
    /// Hydra), split via `Decision::DivideDamage`; a candidate may get none.
    DistributeCountersAmong { what: Selector, total: Value, counter: CounterType },
    /// "Do X to each of up to N targets" (CR 115): slots `0..max_targets`
    /// filtered by `filter`; `effect` runs per supplied target with
    /// `Selector::Target(0)` bound to it (Sea God's Scorn).
    ///
    /// `min_targets` is how many the text requires: 0 = "up to N" (every slot,
    /// slot 0 included, may be declined), 1 = "one or two" (Prismari Charm).
    /// Targets fill left to right; declining ends selection.
    ApplyToTargets {
        max_targets: u8,
        #[serde(default)]
        min_targets: u8,
        filter: SelectionRequirement,
        effect: Box<Effect>,
    },
    /// A trigger its source grants to the object it fired on (Grothama): on the
    /// stack it becomes that object's ability (CR 113.10) — subject as source and
    /// controller, the granter as `TriggerSource` — and runs `0`.
    AsSubjectsAbility(Box<Effect>),
    /// CR 601.2c — a sequence whose members own consecutive target slots (each
    /// starts after the previous member's last, [`Effect::target_slot_count`])
    /// and resolve on their own slice. Suffer the Past is `[player step,
    /// CapTargetsAtX over ApplyToTargets]`. Non-targeting members read the whole
    /// list.
    SlotGroups(Vec<Effect>),
    /// "Up to X targets" (Crackle with Power): truncates the targets to the
    /// cast's `x_value` before `body` (usually `ApplyToTargets` at the static
    /// ceiling). Over-selects at small X are dropped at resolution.
    CapTargetsAtX { body: Box<Effect> },
    /// "X target …" (CR 601.2c — Synod Artificer): like `CapTargetsAtX` but slots
    /// `0..X` are required, so too few targets is rejected rather than
    /// half-firing.
    TargetsExactlyX { body: Box<Effect> },
    /// The `Value`-driven sibling of `CapTargetsAtX`: "up to [amount] target
    /// creatures", where the cap is computed at resolution rather than paid as
    /// {X} (Mogis's Marauder's devotion-to-black cap).
    CapTargetsAt { amount: Value, body: Box<Effect> },
    /// CR 601.2c — "for each opponent, [verb] up to one target X that player
    /// controls": a transparent wrapper over an `ApplyToTargets` capping the
    /// targets at the opponent count and forbidding two with one controller.
    /// A wrapper because slots are declared statically
    /// (`Selector::TargetFiltered { slot, .. }`) and only the game knows the
    /// opponent count. The Primordials, Grasp of Fate.
    ///
    /// ⚠ Every target is "up to", so Sylvan Primordial's mandatory one could be
    /// declined; all are pure upside, so no real seat does.
    ForEachOpponentTarget { body: Box<Effect> },
    /// [`Effect::ForEachOpponentTarget`] over every player, you included (Battle
    /// at the Helvault): capped at the living player count, one per controller.
    ForEachPlayerTarget { body: Box<Effect> },
    /// Transparent wrapper making target slots `>= min` optional ("up to one
    /// target …") for a `body` whose slots come from distinct effects, which
    /// `ApplyToTargets` can't express (it rebinds every target to `Target(0)`).
    /// Primal Might: `min: 1`, the fight's enemy slot may be declined and the
    /// `Fight` no-ops. Only the targeting walk reads it (`min_targets_in_mode` /
    /// `target_slot_optional`).
    OptionalTargets { min: u8, body: Box<Effect> },
    /// Run `body` with X set to `x` (evaluated first), so X-relative filters
    /// (`ManaValueAtMostXFromCost`) read a card-named number — Kodama of the East
    /// Tree: `WithX { x: ManaValueOf(TriggerSource), body }`.
    WithX { x: Value, body: Box<Effect> },
    /// CR 601 — "Until your next turn, your opponents can't cast spells from
    /// anywhere other than their hands" (Avatar's Wrath): a lock the
    /// controller's next untap step lifts.
    OpponentsCantCastFromNonHandUntilYourNextTurn,
    /// CR 121.2a — "the next time [who] would draw a card this turn, instead
    /// they exile the top card of their library. They may play it this turn"
    /// (Urabrask, Heretic Praetor). Spent by that draw; lapses at cleanup.
    NextDrawThisTurnBecomesImpulse { who: PlayerRef },
    /// Run `body` with target slots `slot..slot + count` moved to `0..count`.
    ///
    /// Runtime-only, built by the modal arms' suspend splice: a parked
    /// continuation resumes with the spell's whole target list, so each
    /// remaining mode's slots must be named in the effect or all read slot 0.
    BindTargetSlot {
        slot: u8,
        /// A mode with two targets ("target creature you control fights
        /// target creature you don't control") binds two.
        #[serde(default = "one_u8")]
        count: u8,
        body: Box<Effect>,
    },
    /// Run `body` with `ctx.targets` set to exactly these objects.
    ///
    /// Runtime-only sibling of `BindTargetSlot`, pinning objects the arm picked
    /// itself (an exile-pile card, this iteration's permanent). Without it a
    /// resumed continuation reads the caster's targets.
    BindTargetObjects { ids: Vec<crate::card::CardId>, body: Box<Effect> },
    /// CR 603.2c — "whenever one or more [cards] …, [body] one of them": on a
    /// `once_per_batch` trigger the dispatcher fills `ids` with the batch's
    /// subjects and `body` reads them as targets (`AllTargets` = "them" —
    /// Conspiracy Theorist). Bound, never chosen: no cast-time slot.
    OverTriggerBatch {
        body: Box<Effect>,
        #[serde(default)]
        ids: Vec<crate::card::CardId>,
    },
    /// CR 603.2c — [`Effect::OverTriggerBatch`] for a trigger that also picks its
    /// own targets: `body` reads the batch as [`Selector::BoundTriggerBatch`]
    /// with its slots untouched (Vulpine Harvester). Members are fixed at fire
    /// time; departed ones read LKI.
    WithTriggerBatch {
        body: Box<Effect>,
        #[serde(default)]
        ids: Vec<crate::card::CardId>,
    },
    /// Run `body` with one piece of the resolver's per-iteration `GameState`
    /// scratch (the `VoteTally::PerVote` ballot, the die face) pinned to the
    /// parking iteration's value.
    ///
    /// Runtime-only, the third `Bind*`. A parked continuation resumes after the
    /// loop restored that state, so the splice must name it inside the effect.
    BindScratch { scratch: ScratchBinding, body: Box<Effect> },
    /// Eerie Ultimatum: return any graveyard permanent cards with different names
    /// (`Decision::ChooseCards`; duplicates dropped).
    ReturnGraveyardPermanentsDifferentNames,
    /// Push the Limit: return every [filter] card from `who`'s graveyard under
    /// you; `sacrifice_eot` sacrifices them at the next end step.
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
    /// "You may cast a creature spell from your graveyard this turn"
    /// (Chainer, Nightmare Adept): one cast, any creature card there when
    /// it's cast (`Player::graveyard_creature_cast_turn`).
    MayCastCreatureFromGraveyardThisTurn,
    /// [`Self::MayCastCreatureFromGraveyardThisTurn`] whose cast "enters with
    /// a finality counter on it and is a [creature_type] in addition to its
    /// other types" (The Tomb of Aclazotz's Vampire).
    MayCastCreatureFromGraveyardThisTurnAs { creature_type: crate::card::CreatureType },
    /// Victor, Valgavoth's Seneschal: put a `filter` card from any graveyard onto
    /// the battlefield under you — untargeted `ChooseCards` (auto: priciest).
    PutGraveyardCardOntoBattlefield { filter: SelectionRequirement },
    /// "`who` shuffles up to `max` `filter` cards from their graveyard into their
    /// library" — their `Decision::ChooseCards`, untargeted (Cathartic Parting).
    /// Library mirror of `ReturnGraveyardCardsToHand`.
    ShuffleGraveyardCardsIntoLibrary {
        who: PlayerRef,
        filter: SelectionRequirement,
        max: Value,
        /// Put the chosen cards on top of the library in any order instead of
        /// shuffling them in (Stillness in Motion).
        #[serde(default)]
        to_top: bool,
    },
    /// Genesis Ultimatum: look at the top `count`; chosen permanent cards enter,
    /// the rest to hand.
    LookTopNDeployPermanentsRestToHand { count: Value },
    /// Yidaro, Wandering Monster: on cycling, shuffle it in from the graveyard,
    /// or deploy it after `threshold` cycles of the name
    /// (`GameState.cycled_count_by_name`).
    CycleRecurFromGraveyard { threshold: u32 },
    /// Illuna, Apex of Wishes: exile until a nonland permanent card, then
    /// battlefield or hand (`Decision::OptionalTrigger`); the rest stay exiled.
    ExileTopUntilPermanentToBattlefieldOrHand,
    /// Nethroi, Apex of Death: return graveyard creature cards with total power
    /// ≤ `max_total`, picked at resolution (`Decision::ChooseCards`), accepted
    /// greedily until the next would bust the cap.
    ReturnGraveyardCreaturesUpToTotalPower { max_total: Value },
    /// Scout for Survivors: return up to `max_count` graveyard creature cards
    /// with total mana value ≤ `max_total`, each with `counters` +1/+1 counters;
    /// picks accepted greedily within both caps.
    ReturnGraveyardCreaturesUpToTotalManaValue {
        max_total: Value,
        max_count: Value,
        counters: u32,
    },
    /// Pair o' Dice Lost: return graveyard cards with total mana value ≤
    /// `max_total`, accepted greedily (headless: priciest first).
    ReturnGraveyardCardsToHandUpToTotalManaValue { max_total: Value },
    /// Protean Hulk: the library version of
    /// [`ReturnGraveyardCreaturesUpToTotalManaValue`] — search for creatures with
    /// total mana value ≤ `max_total`, put them onto the battlefield, shuffle.
    SearchLibraryCreaturesUpToTotalManaValue { max_total: Value },
    /// Command the Dreadhorde: choose creature / planeswalker cards in any
    /// graveyard (`Decision::ChooseCards`), take damage equal to their total mana
    /// value, put them onto the battlefield under your control.
    CommandTheDreadhorde,
    /// Swift Silence — counter every other spell on the stack, then draw a
    /// card for each spell countered this way (CR 701.5).
    CounterAllOtherSpellsDrawPer,
    /// "Counter all other spells" — `CounterAllOtherSpellsDrawPer` without
    /// the draw (Reverse the Polarity).
    CounterAllOtherSpells,
    /// Fall (Rise // Fall) — `who` reveals `count` cards at random from their
    /// hand, then discards each nonland card revealed this way. Lands revealed
    /// this way stay in hand.
    RevealRandomDiscardNonland { who: Selector, count: Value },
    /// "Target opponent reveals a card at random from their hand": it stays put,
    /// stamped for `Value::LastRevealedManaValue` and
    /// `Selector::LastRevealedCard` for a later clause.
    RevealRandomFromHand { who: Selector },
    /// Mass Polymorph: exile your creatures, reveal until that many creature
    /// cards, deploy them, shuffle the rest in.
    MassPolymorph,
    /// Wild Evocation: the player reveals a random hand card; a land enters,
    /// otherwise they cast it free (auto-targets) if able.
    RandomHandCardDeployOrCastFree { who: Selector },
    /// "Tap up to N target permanents" with N a runtime `Value` (Archipelagore's
    /// `MutateCount`), chosen at resolution (`Decision::ChooseCards`) instead of
    /// fixed slots. `skip_untap` sets `skip_next_untap`. `exact`: exactly
    /// `count`, an all-or-nothing cost (Aziza; the surrounding `If` guarantees
    /// candidates).
    TapUpToValue { count: Value, filter: SelectionRequirement, skip_untap: bool, exact: bool },
    /// CR 702.110b — emits `GameEvent::Exploited` (Colonel Autumn);
    /// [`shortcut::exploit`] runs it after the sacrifice.
    NoteExploited,
    /// Enlist (CR 702.154): auto-taps the highest-power eligible nonattacking
    /// creature, only when that power is positive, and adds it to this
    /// creature's power until end of turn.
    Enlist,
    /// Enlist (CR 702.154), then — only when a creature was enlisted —
    /// `then` ("if it enlisted a creature this combat, …": Aradesh, the
    /// Founder's payoff on its own attack).
    EnlistThen { then: Box<Effect> },
    /// Create `count` token copies of `source` for `who`, from its printed
    /// definition. `extra_creature_types` are added to the copied subtypes
    /// ("a 0/0 Fractal in addition to its other types" — Applied Geometry), and
    /// `override_pt` replaces P/T before any counters. Tokens are `is_token`, so
    /// the token SBA removes them off the battlefield.
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
        /// CR 707.2 — the copy is exactly these colors ("a 5/5 black Demon" — Ardyn):
        /// a color indicator (CR 105.2c) plus the copied cost's colored pips stripped
        /// (generic kept). `None` keeps the source's colors.
        #[serde(default)]
        override_colors: Option<Vec<crate::mana::Color>>,
        /// The token copy enters tapped (Sin, Spira's Punishment — "create a
        /// tapped token that's a copy"). `false` = enters untapped (default).
        #[serde(default)]
        enters_tapped: bool,
        /// CR 707.9b rider — the token copy isn't legendary (Helm of the
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
        /// "… it has no mana cost" (CR 702.128a embalm, 702.129a eternalize):
        /// the copy's mana cost is empty, so its colors come only from
        /// `override_colors`' indicator and its mana value is 0.
        #[serde(default)]
        no_mana_cost: bool,
        /// "… except it enters with an additional [kind] counter on it"
        /// (Littjara Mirrorlake): placed as part of the mint (CR 614.1c), so
        /// the copy never exists without it.
        #[serde(default)]
        enters_with_counters: Option<(crate::card::CounterType, Value)>,
        /// Keywords the copy loses ("except it has haste and loses soulbond" —
        /// Mirage Phalanx), stripped before it enters.
        #[serde(default)]
        remove_keywords: Vec<crate::card::Keyword>,
    },
    /// "… a Food artifact with '{2}, {T}, Sacrifice: gain 3 life' and loses all
    /// other card types" (Shelob, Child of Ungoliant): each `what` permanent
    /// becomes an artifact — Food — keeping its abilities, written into its
    /// copiable values (CR 707.9b).
    BecomeFoodArtifact { what: Selector },
    /// CR 707.9b — "and it has '[static]'" as a copy exception in `what`'s
    /// copiable values, beside `AmendCopiableValues` (Dollhouse of Horrors).
    AddCopiableStaticAbilities { what: Selector, statics: Vec<crate::card::StaticAbility> },
    /// CR 707.9b — a copy exception written into the copiable values of each
    /// `what`, so its copies carry it: a new name (Mishra's Warform), replaced or
    /// added creature types (The Scarab God, Phantom Steed), supertypes and
    /// colors (Olag).
    AmendCopiableValues {
        what: Selector,
        #[serde(default, with = "crate::static_str_serde::opt")]
        name: Option<crate::static_str_serde::StaticStr>,
        #[serde(default)]
        set_creature_types: Option<Vec<crate::card::CreatureType>>,
        #[serde(default)]
        add_creature_types: Vec<crate::card::CreatureType>,
        /// "… it's legendary … in addition to its other types" (Olag).
        #[serde(default)]
        legendary: bool,
        /// "… blue and black … in addition to its other colors" (Olag).
        #[serde(default)]
        add_colors: Vec<crate::mana::Color>,
        /// "… it's an artifact and it loses all other card types" (Espers to
        /// Magicite): the card types become exactly these; a creature type
        /// line goes with the Creature type (CR 205.3d).
        #[serde(default)]
        set_card_types: Option<Vec<crate::card::CardType>>,
    },
    /// Devastating Onslaught: `count` token copies of `source` for `who`, with
    /// haste until end of turn, sacrificed at the next end step.
    CreateTokenCopiesHasteSac {
        who: PlayerRef,
        count: Value,
        source: Selector,
        /// Exile the tokens at the next end step instead of sacrificing them
        /// (Kindred Charge) — no dies triggers.
        #[serde(default)]
        exile: bool,
    },
    /// Sin, Spira's Punishment — exile a permanent card from `who`'s graveyard
    /// at random, then create a tapped token that's a copy of it. If the
    /// exiled card was a land, repeat (bounded by the graveyard size).
    ExileRandomGraveyardCopyTapped { who: PlayerRef },
    /// CR 701.36 — Populate: `who` creates a token that's a copy of a creature
    /// token they control (their choice; AutoDecider keeps the highest-power
    /// one). No-op if they control no creature token.
    Populate { who: PlayerRef },
    /// CR 717.2 — advance the source Class one level, emitting
    /// `GameEvent::ClassLevelReached`; the "Level N" ability's effect. No-op off
    /// the battlefield.
    AdvanceClassLevel,
    /// "The next time one or more creatures or planeswalkers enter this turn,
    /// they enter as copies of [of]" (Mystic Reflection, CR 707.2 / 614.12). Arms
    /// `GameState::next_entries_copy`, which `apply_enters_as_copy` applies until
    /// the next SBA check.
    NextEntriesEnterAsCopy { of: Selector },
    /// CR 704.5j — "the legend rule doesn't apply to permanents [who]
    /// controls this turn" (Hall of Echoes).
    LegendRuleOffThisTurn { who: PlayerRef },
    /// Detection Tower: this turn your spells and abilities ignore hexproof on
    /// opponents and their creatures.
    IgnoreOpponentHexproofThisTurn,
    /// "Become a legendary Equipment artifact named `name` with equip `equip` and
    /// 'Equipped creature gets +P/+T', losing all other abilities" (The Irencrag
    /// → Everflame), baked over the definition (`CardInstance::bake_grant`) until
    /// it leaves.
    BecomeEquipment {
        what: Selector,
        #[serde(with = "crate::static_str_serde")]
        name: crate::static_str_serde::StaticStr,
        equip: crate::mana::ManaCost,
        power: i32,
        toughness: i32,
    },
    /// CR 707.2 — `what` becomes a copy of `source`: copiable characteristics
    /// overwritten by a clone of its current definition, locked in at resolution
    /// (a one-shot rewrite, like MDFC face-swap, not a layer-1 effect). Instance
    /// state (id, owner, controller, counters, tapped, sickness, token-ness)
    /// stays. `extra_creature_types` adds types (Phantasmal Image). With no
    /// `source` it no-ops, so a 0/0 clone dies to SBA as a declined "you may".
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
        /// CR 707.2 "except its name is [this card's]" — the copier keeps its
        /// own name (Sunfrill Imitator), like `EntersAsCopy::keep_name`.
        #[serde(default)]
        keep_name: bool,
    },
    /// CR 707.9b — Lazav, the Multifarious: `BecomeCopyOf` keeping its own
    /// activated abilities, name and Legendary.
    BecomeCopyKeepingIdentity { what: Selector, source: Selector },
    /// CR 707.9b — "becomes a copy of [source], except it doesn't copy that
    /// creature's color and it has this ability" (Vesuvan Doppelganger): the
    /// copier keeps its current colors and the triggered ability running this.
    BecomeCopyKeepingColorAndThisAbility { what: Selector, source: Selector },
    /// CR 707.9b — Kimahri, Valiant Guardian: `BecomeCopyOf` keeping its
    /// triggered abilities and name, plus `keywords`.
    BecomeCopyKeepingName { what: Selector, source: Selector, keywords: Vec<Keyword> },
    /// CR 707.2 — layer-1 sibling of `BecomeCopyOf`: each `what` is a copy of
    /// `source` for `duration` via `Modification::CopyCardDefinition` (snapshot
    /// at resolution). `non_legendary` strips Legendary (CR 707.9b — Echoing
    /// Equation). `Duration::WhileSourceTapped` watches the copied `source`
    /// (Zygon).
    BecomeCopyOfFor {
        what: Selector,
        source: Selector,
        duration: Duration,
        #[serde(default)]
        non_legendary: bool,
    },
    /// CR 707.9b — Taskmaster, Mercenary Mimic: after a copy, `what`'s copiable
    /// type line is exactly creature with `creature_types` (replaced).
    SetCopyTypeLine { what: Selector, creature_types: Vec<crate::card::CreatureType> },
    /// CR 615 — Sokrates, Athenian Teacher: this turn `what`'s combat damage to a
    /// player is prevented and both draw half of it, rounded down.
    CombatDamageToPlayersBecomesDrawsThisTurn { what: Selector },
    /// CR 707.9a/b — the "except …" of a copy that just ran on `what`, rewriting
    /// the installed copiable values so later copies see it. `keep_activated` /
    /// `keep_triggered` index the copier's printed abilities (Volrath, the
    /// Shapestealer; Mizzium Transreliquat).
    AmendCopy {
        what: Selector,
        #[serde(default)]
        keep_activated: Vec<u8>,
        #[serde(default)]
        keep_triggered: Vec<u8>,
        /// "except it's 7/5" — base power and toughness.
        #[serde(default)]
        pt: Option<(i32, i32)>,
        /// "except its name is [the copier's]".
        #[serde(default)]
        keep_name: bool,
        /// "it's legendary in addition to its other types".
        #[serde(default)]
        legendary: bool,
        #[serde(default)]
        keywords: Vec<Keyword>,
        /// "it's an artifact in addition to its other types" (Saheeli,
        /// Sublime Artificer).
        #[serde(default)]
        card_types: Vec<crate::card::CardType>,
        /// "it's a … Human Villain creature in addition to its other types"
        /// (Absorbing Man) — creature types added to the copied ones.
        #[serde(default)]
        creature_types: Vec<crate::card::CreatureType>,
    },
    /// Target becomes a basic land of `land_type` (losing other types/abilities).
    BecomeBasicLand { what: Selector, land_type: LandType, duration: Duration },
    /// "Target land becomes the basic land type of your choice in addition to
    /// its other types" (Navigator's Compass): the additive sibling of
    /// `LandsBecomeChosenBasicType` — one layer-4 `AddLandType` per land.
    GainChosenBasicLandType { what: Selector, duration: Duration },
    /// Terraformer: choose a basic land type; every `what` land becomes it for
    /// `duration`, losing other types and abilities.
    LandsBecomeChosenBasicType {
        what: Selector,
        duration: Duration,
        /// The controller first picks a *source* basic type and only lands of
        /// that type change (Vision Charm's second mode). The printed card
        /// allows any land type for the source pick; basics are what matter.
        #[serde(default)]
        from_chosen_basic: bool,
    },
    /// "As this enters, choose a basic land type" (via `ChooseColor`, basics
    /// mapping 1:1), stamped on `chosen_land_type` (Realmwright).
    ChooseBasicLandTypeForSource,
    /// Grant `what` landwalk of the source's `chosen_land_type` (stamped by a
    /// preceding `ChooseBasicLandTypeForSource`), permanently. Traveler's
    /// Cloak. No-op when no type was chosen.
    GrantChosenTypeLandwalk { what: Selector },
    /// CR 601.2b — "You and target spell's controller bid life. You start at
    /// 1; in turn order each may top the high bid. The high bidder loses that
    /// much life; if you win, counter that spell." Mages' Contest.
    BidLifeToCounterTargetSpell { what: Selector },
    /// CR 305 — each permanent gains all five basic land types for `duration`
    /// (additive layer-4 `AddLandType`). Energybending.
    GainAllBasicLandTypes { what: Selector, duration: Duration },
    /// CR 305 — each resolved permanent *gains* one land type in addition to
    /// its other types (layer 4, additive). Sealock Monster's monstrosity
    /// trigger ("target land becomes an Island in addition to its other types").
    GainLandType { what: Selector, land_type: crate::card::LandType, duration: Duration },
    /// Becomes a creature with the given P/T and types, losing all other types,
    /// abilities and subtypes (CR 613 layers 4/6/7): Oko's Elk. Default 1/1.
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
    /// CR 603 — the EXO Oath template: at each upkeep, that player picks an
    /// opponent leading them on `tally`; `body` runs as the upkeep player with
    /// the opponent in slot 0. Multiplayer auto-picks the biggest lead.
    OathCatchUp { tally: crate::card::PlayerTally, body: Box<Effect> },
    /// Move every counter of `kind` from each resolved `from` onto `to` (Spike
    /// Cannibal). Relocation, not creation — no doublers (CR 122.5).
    MoveAllCountersOfKind { from: Selector, to: Selector, kind: crate::card::CounterType },
    /// "Move a counter of each kind not on [to] from [from] onto [to]"
    /// (Goldberry, River-Daughter). One counter per kind `from` carries and
    /// `to` lacks; relocation, not creation (CR 122.5).
    MoveOneOfEachAbsentKind { from: Selector, to: Selector },
    /// Target a `filter` player (slot 0), then `then`, for bodies that don't
    /// reference the target — EXO Keepers' "target opponent who <trails you>".
    TargetPlayerThen { filter: crate::card::SelectionRequirement, then: Box<Effect> },
    /// Attach `what` (Aura/Equipment) to `to`.
    Attach { what: Selector, to: Selector },
    /// Licid: becomes an Aura with enchant creature attached to `host`; pay
    /// `end_cost` to end it. The creature definition is stashed and the live one
    /// is an Aura with one `LicidDetach` ability; the riders sit on the printed
    /// `equipped_bonus`, inert until attached.
    LicidAttach { host: Selector, end_cost: crate::mana::ManaCost },
    /// The `end_cost` half of [`Effect::LicidAttach`] — the source unattaches
    /// and goes back to being a creature.
    LicidDetach,

    // ── Stack interaction ────────────────────────────────────────────────────
    /// Counter target spell (removes from stack; sends to owner's graveyard).
    CounterSpell { what: Selector },
    /// CR 115.7 — change the target of a spell with a single target: the
    /// controller repoints it (current target offered first). Spells with zero or
    /// several targets are left alone. Shunt.
    ChangeSpellTarget { what: Selector },
    /// Meddle — `ChangeSpellTarget` when the single target is a creature,
    /// and only to another creature.
    ChangeSpellCreatureTarget { what: Selector },
    /// Test of Talents: counter the spell, exile every same-named card from its
    /// controller's graveyard, hand and library; they shuffle and draw one per
    /// card exiled from hand.
    CounterSpellExileSameNamed { what: Selector },
    /// Counter target spell; if the mana spent to cast it was less than its
    /// mana value, the controller draws a card (Unravel).
    CounterSpellDrawIfUnderpaid { what: Selector },
    /// Counter target spell into `zone` instead of the graveyard (CR 701.6a's
    /// default): library top (Memory Lapse), exile (Spell Crumple), hand
    /// (Remand).
    CounterSpellToZone {
        what: Selector,
        zone: CounteredSpellZone,
    },
    /// Move a target spell like `CounterSpellToZone` without countering (CR
    /// 701.6a), so uncounterable spells move too (Reprieve, Spell Queller).
    MoveSpellToZone {
        what: Selector,
        zone: CounteredSpellZone,
    },
    /// Ashiok's Erasure: counter and exile the spell linked to the source, and
    /// stamp its `named_card` so `OpponentsCantCastNamed` locks the name.
    CounterSpellExileNameLock { what: Selector },
    /// Counter an activated or triggered ability: removes the topmost
    /// `StackItem::Trigger` from the `what` source (Consign to Memory).
    CounterAbility { what: Selector },
    /// "Counter target activated ability from an artifact source and destroy
    /// that artifact if it's on the battlefield" (Ouphe Vandals) — the
    /// `CounterAbility` sibling that also kills the source.
    CounterAbilityAndDestroySource { what: Selector },
    /// Tishana's Tidebinder: counter up to one ability; an artifact, creature or
    /// planeswalker source loses all abilities while this remains.
    CounterAbilityAndStripSource { what: Selector },
    /// Voidslime: counter a spell (by card id) or ability (by source, like
    /// `CounterAbility`). Uncounterable spells are skipped.
    CounterSpellOrAbility { what: Selector },
    /// Counter the spell unless its controller pays `mana_cost`, auto-paid on
    /// their behalf (Mystical Dispute). `uncounterable` spells are skipped.
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
        /// "…unless its controller pays {2}. If they do, surveil 2" (Don't Make a
        /// Sound): runs only when paid, in this effect's context (`You` = the
        /// counterspell's controller). As [`Effect::UnlessPlayerPays`]'s `if_paid`.
        #[serde(default)]
        if_paid: Option<Box<Effect>>,
    },
    /// CR 702.21 Ward: counter the topmost stack item from `target` (spell or
    /// trigger) unless its controller pays `cost` (auto-paid). Unlike
    /// `CounterUnlessPaid` it counters abilities too (CR 702.21a) and takes a
    /// `WardCost` (mana / life / discard / sacrifice).
    CounterUnless {
        what: Selector,
        cost: crate::card::WardCost,
    },
    /// "Unless [who] pays [cost], [then]" — the Rhystic tax (Rhystic Study,
    /// Smothering Tithe). `who` (usually `Triggerer`) is asked; declining or
    /// unaffordable runs `then` in the trigger's context (`You` = the rider's
    /// controller). AutoDecider declines. No `who`: `then` runs outright.
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
    /// "Copy it, except the copy isn't legendary" (The Sixth Doctor): one
    /// copy of the spell `what` names, with Legendary stripped (CR 707.9b),
    /// so a permanent copy survives the legend rule beside the original.
    CopySpellNonLegendary { what: Selector },
    /// `CopySpell` whose permanent-spell copies get riders: `grant_haste` and
    /// `sacrifice_eot` (Choreographed Sparks). Stamped on the copy's
    /// `CardInstance.resolve_riders`, applied by the permanent path in
    /// `stack.rs`; no-op on instants and sorceries.
    CopySpellWithRiders {
        what: Selector,
        count: Value,
        grant_haste: bool,
        sacrifice_eot: bool,
        /// CR 707.10c — "you may choose new targets for the copy" (Nalfeshnee
        /// copying an Aura spell).
        #[serde(default)]
        may_choose_new_targets: bool,
    },
    /// CR 707.9b — copy the spell as a 1/1 `creature_type` creature in addition
    /// to its other types, kept by the token it becomes (Donal's Spirit,
    /// Salinae's Fish; an artifact spell's copy becomes an artifact creature).
    CopySpellAsOneOne { what: Selector, creature_type: crate::card::CreatureType },
    /// CR 707.9b — copy the spell `what` once, "except it isn't legendary if
    /// the spell is legendary" (Double Major); the token it resolves into
    /// keeps the exception.
    CopySpellNotLegendary { what: Selector },
    /// CR 707.9b — copy the spell `what` once, "except the copy is [types] in
    /// addition to its other types" (Tawnos, the Toymaker's artifact copy);
    /// the token it resolves into keeps the exception.
    CopySpellAddingTypes { what: Selector, types: Vec<crate::card::CardType> },
    /// CR 701.50 — each `what` permanent connives `amount`: draw and discard that
    /// many, a +1/+1 counter per nonland card discarded this way (Kamiz).
    /// `shortcut::connive`, the self-only form, counts every discard of the
    /// resolution.
    Connive { what: Selector, amount: Value },
    /// Gogo: copy the ability from the `what` source (as `CounterAbility`)
    /// `times` times; each copy is the copier's and may retarget (CR 707.10c).
    CopyAbility { what: Selector, times: Value },
    /// Wandering Archaic: the caster of `what` may pay `mana_cost`
    /// (`Decision::OptionalTrigger`); if they don't or can't, copy the spell
    /// `count` times. AutoDecider declines.
    CopySpellUnlessPaid {
        what: Selector,
        mana_cost: crate::mana::ManaCost,
        count: Value,
    },
    /// Copy target spell `count` times, each copy may be retargeted (CR 707.12 +
    /// 115.7) via `Decision::ChooseTarget`, original offered first so the
    /// AutoDecider keeps it. Twincast.
    CopySpellMayChooseTargets { what: Selector, count: Value },
    /// Copy the spell `what` names under `who`'s control; they may choose new
    /// targets (CR 707.10c, 115.7). "Target opponent may also copy that
    /// spell" (Parnesse, the Subtle Brush) under a `PlayersMayAccept`.
    CopySpellForPlayer { what: Selector, who: PlayerRef },
    /// CR 706 — Onslaught Chain rider: `who` may copy this spell and retarget it,
    /// so the chain bounces around the table. `cost` is the toll paid first; a
    /// declined or unpayable toll ends it. Asked through the installed decider;
    /// a per-seat UI prompt is a follow-up (TODO.md decision-plumbing audit).
    MayCopyThisSpell { who: PlayerRef, cost: ChainCopyCost },
    /// CR 702.18 — Autumn Willow: `player` ignores the source's shroud this turn.
    WaiveShroudForPlayerThisTurn { player: PlayerRef },
    /// "Destroy each [filter] unless its controller pays `life` life" — one
    /// pay-or-die decision per permanent, asked of its controller (Giant
    /// Albatross). `no_regen` denies regeneration (CR 701.19).
    DestroyEachUnlessPaysLife {
        filter: SelectionRequirement,
        life: u32,
        #[serde(default)]
        no_regen: bool,
    },
    /// Zada, Hedron Grinder: a spell targeting only this is copied per other
    /// creature you control it could target. No-op if it targets anything else.
    CopyForEachOtherTargetableCreature,
    /// Feather, Radiant Arbiter: pay `per_copy` per other creature the spell
    /// could target to copy it onto each.
    MayPayToCopyOntoOtherCreatures { per_copy: crate::mana::ManaCost },
    /// CR 115.7d — "you may choose new targets for target spell": each declared
    /// slot repointed in place against its own filter, chosen by this effect's
    /// controller (Redirect). Unlike `CopySpellMayChooseTargets`, mutates the
    /// original.
    ChooseNewTargetsForSpell { what: Selector },
    /// "You may choose new targets for any number of other spells and/or
    /// abilities" (Boltbender, CR 115.7d): untargeted, so the controller's
    /// pick of which ones — `game/retarget_any.rs` names its policy.
    ChooseNewTargetsForAnyNumber,
    /// "Gain control of target spell" (Aethersnatch). The spell's controller
    /// becomes this effect's controller (CR 110.2, 608.2 — a permanent spell
    /// enters under it); its owner is unchanged.
    GainControlOfSpell { what: Selector },
    /// Psychic Battle — each player reveals the top card of their library; the
    /// player who revealed the greatest mana value (uniquely) may repoint the
    /// targets of the spell that just chose them. A tie changes nothing.
    RevealTopGreatestMayChangeTargets,
    /// Demonstrate (CR 702.144): copy for the caster, then an opponent copies it
    /// too; each copy may retarget. Always demonstrates (copying a beneficial
    /// spell is ~always right). Driven by a `SpellCast`/`SelfSource` trigger so
    /// both copies land above the original.
    Demonstrate,
    /// "Exile this face down. It becomes foretold" (The Foretold Soldier,
    /// CR 702.143): the source leaves for exile face down, keeping its
    /// printed foretell cost, and counts as foretold this turn.
    ExileSelfForetold,

    // ── Cast-without-paying / may-play ───────────────────────────────────────
    /// "You may play those cards [duration]. If you cast a spell this way, pay
    /// life equal to its mana value" (Inside Information): a
    /// `MayPlayPermission` with `pay_life`.
    GrantMayPlayForLife { what: Selector, duration: crate::card::MayPlayDuration },
    /// "Until [duration], you may cast/play that card": stamps
    /// `CardInstance.may_play_until` on each `what` card, cast via
    /// `GameAction::CastFromZoneWithoutPaying` at its normal timing. `to_owner`
    /// grants the card's owner instead (Suspend Aggression). `exile_after` sends
    /// cast instants / sorceries to exile; permanents ignore it.
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
    /// Chained after `GrantMayPlay`: a cast through the permission stops its
    /// caster casting more spells this turn. No shipped card uses it (Conduit of
    /// Worlds casts as it resolves, per its 2023-02-04 ruling).
    LockSpellsAfterGrantedCast { what: Selector },
    /// After `GrantMayPlay`: a spell cast through it goes to the library bottom
    /// instead of the graveyard (Quintorius, Loremaster).
    BottomInsteadAfterGrantedCast { what: Selector },
    /// After `GrantMayPlay`: what is played through it enters tapped (Kaldring,
    /// the Rimestaff; CR 614.1c).
    EntersTappedAfterGrantedPlay { what: Selector },
    /// Mark the may-play permission on each card `what` resolves to (chain
    /// after `GrantMayPlay`): "if you cast a spell this way, it has affinity
    /// for artifacts" (Chiss-Goria, Forge Tyrant; CR 702.41a).
    AffinityAfterGrantedCast { what: Selector },
    /// Mark the may-play permissions on each card `what` resolves to as one
    /// group (chain after `GrantMayPlay`): "you may cast a spell from among
    /// them" — the first cast through any of them consumes the rest.
    OneCastAmongGranted { what: Selector },
    /// "When you play a card this way, [body]" (Fires of Mount Doom), after a
    /// `GrantMayPlay`: each card is registered for the turn, and playing it
    /// pushes `body` as the source's trigger (CR 603.2) with it as subject.
    WhenPlayedThisWayThisTurn { what: Selector, body: Box<Effect> },
    /// The "up to `plays`" sibling of `OneCastAmongGranted` (March of Reckless
    /// Joy's "you may play up to two of those cards"): the grants form one
    /// group that `plays` plays spend (`ColdState.group_plays_left`).
    PlaysAmongGranted { what: Selector, plays: u32 },
    /// Mark the may-play permission on each card `what` resolves to as "you
    /// may **cast**": a land among them can't be played (Ian Malcolm,
    /// Chaotician). Chain after `GrantMayPlay`.
    GrantedCastOnly { what: Selector },
    /// Once-each-turn `OneCastAmongGranted` (Maralen, Fae Ascendant): the grants
    /// are void once a cast through the group happened this turn
    /// (`TurnRegistries.one_cast_groups_spent_this_turn`).
    OneCastPerTurnAmongGranted { what: Selector },
    /// CR 601.2 — a mana value cap on granted casts read at cast time, not at
    /// grant (Maralen's Elf/Faerie count): chained after the grant, stored in
    /// `TurnRegistries.granted_cast_mv_caps`.
    CapGrantedCastsAtCast { what: Selector, max_mana_value: Value },
    /// After `GrantMayPlay`: "it costs [cost] more this way unless it targets a
    /// [filter]" (Mavinda, Students' Advocate) — in
    /// `CardInstance.granted_cast_surcharge_eot`, checked at target validation.
    StampMayPlaySurcharge {
        what: Selector,
        cost: crate::mana::ManaCost,
        filter: crate::card::SelectionRequirement,
    },
    /// A one-shot permission to cast `what`'s MDFC back face from the graveyard
    /// (`may_cast_back_from_graveyard`, consumed by `GameAction::CastSpellBack`).
    /// No-op without a back face.
    ///
    /// ⚠ No card builds this: it was written for a Pestilent Cauldron with
    /// fabricated oracle text. The printed shape is Disturb (CR 702.145), a
    /// keyword reaching `CastSpellBack` through `Keyword::Disturb(_)`. The flag
    /// stays as a capability; only this writer is orphaned.
    GrantCastBackFromGraveyard { what: Selector },
    /// Resolve-now `GrantMayPlay`: asks "cast `what` free?"
    /// (`Decision::OptionalTrigger`); on yes it is free-cast from `source_zone`
    /// with auto-targets. Improvisation Capstone, The Dawning Archaic.
    CastWithoutPayingImmediate {
        what: Selector,
        source_zone: crate::card::Zone,
        #[serde(default)]
        exile_after: bool,
        /// Cast a *copy* of the card instead — the original stays put
        /// (Capricious Hellraiser, CR 707.12).
        #[serde(default)]
        copy: bool,
        /// Non-zero: not free — pay the card's cost minus this much generic
        /// (God-Eternal Kefnet); declining or unpayable skips the cast.
        #[serde(default)]
        reduce_generic: u32,
        /// The cast isn't free at all — the controller pays the card's own
        /// cost (minus `reduce_generic`). "Copy them. You may cast any number
        /// of the copies" (The Tale of Tamiyo IV).
        #[serde(default)]
        pay_own_cost: bool,
    },
    /// Bismuth Mindrender: may cast it now for life equal to its mana value (CR
    /// 119.4; not later in the turn per the 2024-06-07 ruling).
    CastImmediateForLife { what: Selector, source_zone: crate::card::Zone },
    /// "You may cast [it], and mana of any type can be spent to cast that
    /// spell" as the effect resolves (Tinybones, the Pickpocket): its own cost,
    /// paid as generic mana equal to its mana value (CR 609.4b).
    CastImmediateAnyManaType { what: Selector, source_zone: crate::card::Zone },
    /// "You may cast any number of spells from among them free": re-offers the
    /// remaining cards after each accepted cast, so the controller picks the
    /// order; ends after a pass with no accepts. Lands skipped. `filter` narrows
    /// it (Epic Experiment); `None` = any nonland.
    CastAnyOrderWithoutPaying {
        what: Selector,
        source_zone: crate::card::Zone,
        #[serde(default)]
        filter: Option<SelectionRequirement>,
        /// Cap on how many may be cast ("a spell from each opponent's
        /// graveyard" — Jetsam). `None` is the printed "any number".
        #[serde(default)]
        cap: Option<Value>,
        /// Cap on the *total* mana value of what is cast ("spells … with
        /// total mana value X or less" — Rod of Absorption). A card whose
        /// mana value would overrun what is left is skipped.
        #[serde(default)]
        total_mana_value: Option<Value>,
        /// "If that spell would be put into a graveyard, exile it instead"
        /// (Jetsam, Sorcerous Squall).
        #[serde(default)]
        exile_after: bool,
    },
    /// "You may cast a [filter] spell from your hand free" (Maelstrom Archangel):
    /// one nonland hand card via `Decision::ChooseCards` (AutoDecider declines),
    /// cast with auto-targets.
    CastFromHandWithoutPaying {
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },
    /// Anrakyr the Traveller: cast one `filter` spell from hand or graveyard for
    /// life equal to its mana value, during resolution (CR 119.4).
    MayCastFromHandOrGraveyardForLife { filter: SelectionRequirement },
    /// "Each [filter] card in your graveyard gains unearth {cost} until end of
    /// turn" (Ghost Ark) — the one-shot twin of
    /// `StaticEffect::GraveyardCardsHaveUnearth`.
    GraveyardCardsGainUnearthThisTurn { filter: SelectionRequirement, cost: crate::mana::ManaCost },
    /// CR 702.104 Tribute N: an opponent may put N +1/+1 counters on the entering
    /// source (`Decision::OptionalTrigger`; AutoDecider declines), else
    /// `otherwise`. Built by `shortcut::tribute`.
    Tribute { n: u32, otherwise: Box<Effect> },
    /// Paradigm (SOS): a recurring `DelayedKind::YourNextMainPhase` trigger
    /// offering "cast a copy of this from exile free?" each precombat main
    /// (`Effect::CastFreeParadigmCopy`). Trails the Paradigm Lessons, with
    /// `exile_on_resolve = true`.
    RegisterParadigm,
    /// Paradigm body: finds the exiled source and, on yes, free-casts a token
    /// copy of it; the original stays for the next main phase.
    CastFreeParadigmCopy,

    /// Cipher (CR 702.99): may encode this spell on your creature
    /// (`CardInstance.encoded_on`), which casts a free copy on each combat
    /// damage to a player. A pending flag routes the card to exile in
    /// `continue_spell_resolution`.
    Cipher,

    /// Cascade (CR 702.85): exile from the top until a nonland card with mana
    /// value < `max_mv`, may cast it free; the rest go to the bottom (unordered,
    /// like `RevealMissDest::BottomRandom`).
    ///
    /// `max_mv` is the printed mana value (CR 702.85b ignores cost reduction);
    /// [`cascade`] wires the trigger. `filter` narrows the stop (Jodah, the
    /// Unifier).
    Cascade {
        max_mv: Value,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<SelectionRequirement>,
    },
    /// CR 702.60 Ripple N: reveal the top N, cast any same-named cards free,
    /// bottom the rest; rippled copies ripple again via the cast path. [`ripple`]
    /// wires the trigger. `of_trigger_spell`: the trigger's source spell ripples
    /// (Thrumming Stone).
    Ripple {
        n: Value,
        #[serde(default)]
        of_trigger_spell: bool,
    },
    /// "Reveal the top card; you may cast it free if it matches `filter`" — a
    /// declined or unmatched card stays on top (Powerbalance, via `WithX`).
    RevealTopMayCastFreeKeepOnTop { filter: SelectionRequirement },
    /// Reveal until a nonland card with mana value ≤ `max_mv` (and `filter`), may
    /// cast it free, bottom the rest at random — revealed, not exiled (Treasure
    /// Keeper).
    RevealUntilMayCastFree {
        max_mv: Value,
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },
    /// Sunbird's Invocation: reveal the top `count`, cast at most one spell with
    /// mana value ≤ `max_mv` free (CR 601.2), bottom the rest. Lands skipped.
    RevealTopMayCastOneFree {
        count: Value,
        max_mv: Value,
        /// Only cards matching this may be cast. `None` allows any nonland
        /// card.
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },
    /// "Each player draws cards equal to the amount of damage dealt to [the
    /// source] this turn by sources they controlled." (Grothama,
    /// All-Devouring's leaves-the-battlefield trigger.)
    EachPlayerDrawsDamageTheyDealtToSource,
    /// Exterminator Magmarch: if another opponent controls a nonland permanent
    /// the spell (`TriggerSource`) could target, copy it onto one; the copy is
    /// the copier's (CR 707.10c).
    CopySpellOntoAnotherOpponentsPermanent { what: Selector },
    /// Zevlor, Elturel Exile: per other opponent, copy the spell onto that player
    /// or one of their permanents (CR 707.10c).
    CopySpellForEachOtherOpponent { what: Selector },

    /// Exile the top `count` cards of `who`'s library, each with a may-play for
    /// `duration` (Ark of Hunger; Fallen Shinobi's two) — one composite so the
    /// permission hits the moved cards.
    ExileTopAndGrantMayPlay {
        who: PlayerRef,
        count: Value,
        duration: crate::card::MayPlayDuration,
        /// The cast pays the card's cost as generic (Nassari, Dean of Expression).
        /// With `pay_own_cost` it is "mana of any COLOR": own cost, colored pips
        /// generic (CR 609.4b; {C} and snow kept). `false` keeps the free cast (Urza,
        /// Lord High Artificer).
        #[serde(default)]
        pay_any_color: bool,
        /// Only cards at or under this mana value get the permission — the
        /// rest stay exiled with no grant (Kotis, the Fangkeeper's "spells
        /// with mana value X or less"). `None` grants on every exiled card.
        #[serde(default)]
        max_mana_value: Option<Value>,
        /// "You may play them" paying their own cost — plain impulse draw (Light Up
        /// the Stage). Exclusive with `pay_any_color`. ⚠ The `false` free-cast
        /// default is right only for "without paying its mana cost";
        /// `scripts/audit_free_impulse.py` checks the class.
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
        token: std::sync::Arc<TokenDefinition>,
    },

    /// Gandalf, Goblins' Bane: exile the top `count` face down; play them while
    /// exiled if you control a `creature_type`, at their own cost.
    ExileTopFaceDownPlayIfYouControl { count: Value, creature_type: crate::card::CreatureType },
    /// Gonti, Night Minister: exile the top of `library`'s library face down;
    /// `grantee` may play it while exiled, any mana type spendable (its mana
    /// value as generic, CR 609.4b). `spend` narrows: own cost (Mindleecher), or
    /// "any color" keeping {C} pips (Rogue Class).
    ExileTopFaceDownGrantPlay {
        library: PlayerRef,
        grantee: PlayerRef,
        #[serde(default)]
        spend: ExiledPlaySpend,
        /// "You may **cast** that card" — a land can't be played (Rev, Tithe
        /// Extractor).
        #[serde(default)]
        cast_only: bool,
    },

    /// CR 614 — as this enters, exile up to `count` `filter` graveyard cards
    /// stamped `exiled_with = source` (Mimeoplasm, Revered One).
    AsEntersExileFromYourGraveyard { count: Value, filter: SelectionRequirement },

    /// CR 707.2 — become a copy of an exiled-with card, `base_pt` and keeping the
    /// granting ability so it can copy again (Mimeoplasm, Revered One).
    BecomeCopyOfExiledCard { what: Selector, base_pt: Option<(i32, i32)> },

    // ── Sacrifice ────────────────────────────────────────────────────────────
    Sacrifice { who: Selector, count: Value, filter: SelectionRequirement },
    /// "Each player who controls the most [filter] sacrifices `count`
    /// [filter]" (Tectonic Hellion). The tied-for-most set is fixed before
    /// anyone sacrifices.
    PlayersWithMostSacrifice { filter: SelectionRequirement, count: Value },
    /// Each player picked by `who` sacrifices **all** permanents they control
    /// matching `filter` — no choice involved (CR 701.16). All Is Dust's
    /// "each player sacrifices all colored permanents they control".
    SacrificeAllMatching { who: Selector, filter: SelectionRequirement },
    /// Sacrifice each permanent `what` resolves to, by its own controller
    /// (CR 701.16). The targeted sibling of `SacrificeAllMatching`.
    SacrificeSelected { what: Selector },
    /// CR 701.21 — "Each player sacrifices all [`filter`] they control except
    /// for `keep`" (Keldon Firebombers). Each player keeps their `keep`
    /// highest-mana-value matches; the rest go.
    EachPlayerSacrificesDownTo { filter: SelectionRequirement, keep: Value },
    /// Living Death: each player exiles graveyard creatures, sacrifices their
    /// creatures, then deploys the exiled cards.
    LivingDeath,
    /// Vito, Fanatic of Aclazotz: runs `modes[min(n, len-1)]` for the n-th
    /// resolution this turn (keyed by source), then counts it.
    EscalatingThisTurn { modes: Vec<Effect> },
    /// Bringer of the Last Gift: everyone sacrifices their other creatures, then
    /// returns the creature cards already in their graveyard.
    SacrificeOthersThenReanimate,
    /// Show and Tell: each player (APNAP) may put a `filter` permanent card from
    /// hand onto the battlefield; `others_only` skips the controller (Hunted
    /// Wumpus). Every seat auto-picks its priciest; the "may" isn't prompted yet
    /// (TODO.md).
    EachPlayerMayPutPermanentFromHand {
        filter: SelectionRequirement,
        #[serde(default)]
        others_only: bool,
        /// Eureka — "repeat this process until no one puts a card onto the
        /// battlefield". Loops the whole APNAP pass while any seat played.
        #[serde(default)]
        repeat: bool,
    },
    /// "`who` exiles `count` permanents of their choice" — exile Annihilator
    /// (Bane of Bala Ged). UI prompts on a real choice; else the weakest.
    PlayerExilesPermanents { who: PlayerRef, count: Value, filter: SelectionRequirement },
    /// "`who` returns `count` permanents of their choice to hand" — bounce
    /// analogue of `PlayerExilesPermanents` (Devastating Mastery). `up_to` makes
    /// it optional; auto returns the weakest.
    PlayerReturnsPermanentsToHand {
        who: PlayerRef,
        count: Value,
        filter: SelectionRequirement,
        up_to: bool,
    },
    /// CR 701.16 — sacrifice this effect's source, with death triggers (Ball
    /// Lightning); a `Move` to the graveyard would skip `CreatureDied`.
    SacrificeSource,
    /// Exile this effect's source permanent (no death trigger). Used by
    /// temporary tokens exiled at the next end step (Manaform Hellkite's
    /// Dragon Illusion, Kari Zev's Ragavan-style temps).
    ExileSource,
    /// CR 701.16 — sacrifice the specific permanents `what` names (each by its
    /// controller), unlike `Effect::Sacrifice`'s player choice: a delayed
    /// end-step sacrifice (Footsteps of the Goryo).
    SacrificePermanent { what: Selector },
    /// "Sacrifice this unless you sacrifice [count] [filter]" (The Gitrog
    /// Monster): with too few, or a UI decline, the source goes. AutoDecider
    /// keeps it by sacrificing the weakest matches.
    SacrificeSourceUnlessSacrifice {
        filter: SelectionRequirement,
        #[serde(default = "one_u32")]
        count: u32,
    },
    /// "Sacrifice this permanent unless you return a [`filter`] you control to
    /// its owner's hand" (the Invasion-block Lair lands). The bounce sibling of
    /// `SacrificeSourceUnlessSacrifice`.
    SacrificeSourceUnlessReturn { filter: SelectionRequirement },
    /// "Sacrifice this unless you pay [cost]" over [`crate::card::WardCost`]
    /// (Rotting Giant). UI gets yes/no; AutoDecider pays whenever it can.
    SacrificeSourceUnlessCost { cost: crate::card::WardCost },
    /// Plunge into Darkness: sacrifice any number of [filter]
    /// (`Decision::ChooseAmount`, auto 0); `per_each` runs per sacrifice.
    SacrificeAnyNumber {
        who: PlayerRef,
        filter: SelectionRequirement,
        per_each: Box<Effect>,
    },
    /// Plunge into Darkness: pay X life (≤ life; auto 0), look at X, take one,
    /// exile the rest.
    PayLifeLookTake { who: PlayerRef },
    /// Vizkopa Confessor: pay N life (`Decision::ChooseAmount`, auto 0); `opp`
    /// reveals their cheapest N (modeling their choice) and you exile one. `opp`
    /// is `EachOpponent`, exact in a duel.
    PayLifeRevealExileFromHand { opp: PlayerRef },
    /// "You may pay any amount of life. If you do, draw that many cards."
    /// Amount via `Decision::ChooseAmount`, capped at current life
    /// (AutoDecider pays 0). Necrodominance's end step.
    PayLifeDraw { who: PlayerRef },
    /// CR 701.30 — clash with the most hostile opponent (or `with` — Marvo): both
    /// reveal the top card and may bottom it (APNAP); the winner gets
    /// `GameEvent::ClashWon`, and if it's you `on_win` runs with them as
    /// `ChosenPlayerOfSource` (Pollen Lullaby).
    ClashWithOpponent {
        on_win: Box<Effect>,
        /// "Otherwise, …" — runs when the controller doesn't win (Pulling
        /// Teeth, Captivating Glance).
        #[serde(default)]
        on_lose: Option<Box<Effect>>,
        #[serde(default)]
        with: Option<PlayerRef>,
    },
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
    EachPlayerRevealsCreaturesForTokens { token: std::sync::Arc<crate::card::TokenDefinition> },

    /// False Cure: this turn, every player loses `per` per life gained — unlike
    /// the controller-scoped [`Effect::WheneverYouGainLifeThisTurn`].
    AnyLifeGainPunishedThisTurn { per: u32 },

    /// General [`Effect::RevealUntilNonlandDamage`]: reveal until a nonland card,
    /// bottom them at random, then `then` with its mana value as
    /// [`Value::LastRevealedManaValue`]. Kaboom!.
    RevealUntilNonlandThen { then: Box<Effect> },

    /// "Choose a creature type, then [`then`]" — the resolution-time
    /// [`Effect::NameCreatureType`], stamping the source's
    /// `chosen_creature_type` for `then` (Riptide Shapeshifter).
    ChooseCreatureTypeThen { who: PlayerRef, then: Box<Effect> },

    /// "Each player chooses a creature type. [`then`]": APNAP; the union is
    /// `SelectionRequirement::IsTypeChosenThisWay` (Patriarch's Bidding).
    EachPlayerChoosesCreatureTypeThen {
        then: Box<Effect>,
        /// Run `then` once per player instead, as that player (`You`) with
        /// only their own pick published — "each player chooses a creature
        /// type and returns … of that type" (Grave Sifter).
        #[serde(default)]
        per_player: bool,
    },
    /// "Choose an opponent. [then]" — not a target (CR 115.10): stamped on the
    /// source's `chosen_player` for `ChosenPlayerOfSource`. A seat ballot;
    /// headless picks the fewest creatures, turn order breaking ties (Sylvan
    /// Offering).
    ChooseOpponentThen { then: Box<Effect> },
    /// [`Effect::ChooseOpponentThen`] whose ballot (and headless pick) is led
    /// by the opponent `lead` names (Sandstone Oracle: the most cards in hand).
    ChooseOpponentLedBy { lead: PlayerRef, then: Box<Effect> },
    /// "Choose an opponent at random. [body]" with one pick for the whole
    /// body (Truth or Consequences: one opponent takes every consequences
    /// vote's damage), read back by `PlayerRef::ChosenPlayerOfSource`.
    WithRandomOpponent { body: Box<Effect> },
    /// Comet, Stellar Pup: `amount` damage to a creature or player chosen at
    /// resolution, untargeted (2022-10-07 ruling). Headless: the strongest
    /// enemy it kills, else the lowest-life opponent.
    DealDamageToChosenCreatureOrPlayer { amount: Value },
    /// CR 706.6 — "roll a d20 for each [thing] and ignore all but the highest
    /// roll" (Iron Mastiff): `roll` rolls `dice` dice (at least one) and
    /// ignores every one but the highest.
    IgnoreAllButHighestRoll { dice: Value, roll: Box<Effect> },
    /// "You may activate [this]'s loyalty ability two more times this turn"
    /// (Comet, Stellar Pup's 6), CR 606.3.
    LoyaltyTwoMoreTimesThisTurn { what: Selector },
    /// `amount` to [that player] or one of their planeswalkers, chosen at
    /// resolution (headless: a walker it kills, else the player). Vial Smasher.
    DealDamageToPlayerOrPlaneswalker { who: PlayerRef, amount: Value },
    /// [`Effect::WithRandomOpponent`] over every living player, you included
    /// ("choose a player at random" — Wildfire Devils).
    WithRandomPlayer { body: Box<Effect> },
    /// Continuation of a suspended "cast the exiled card free?" offer (Cascade,
    /// Discover) for a prompting seat: on yes `player` casts `card`, target first
    /// (CR 601.2c). Engine-built only.
    CastExiledFreeAsking { card: crate::card::CardId, player: usize },
    /// Hildibrand Manderville: may cast the dead source from your own graveyard
    /// as an Adventure until the end of your next turn.
    GrantAdventureFromGraveyard,
    /// "Do this only once each turn": spends the source's use where the action
    /// happens (inside `MayDo`'s yes), so declining keeps it
    /// (`Predicate::SourceDoneThisTurn`).
    MarkDoneThisTurn,
    /// "That player [does body]": `body` with `who` as controller (Skullwinder).
    /// No-op if `who` is nobody or has left.
    AsPlayer { who: PlayerRef, body: Box<Effect> },
    /// Reveal until `count` lands; they enter (`tapped`), the rest bottom at
    /// random; stops at the library's end (Skyserpent Seeker).
    RevealUntilLandsToBattlefield { count: Value, tapped: bool },
    /// Reality Scramble: reveal until a card sharing a card type with `with`
    /// (read as this resolves; may be the revealed card, CR 701.20), put it onto
    /// the battlefield, bottom the rest at random. A `with` with no card type
    /// reveals nothing.
    RevealUntilSharesCardTypeToBattlefield { with: Selector },
    /// "Reveal until `count` `filter` cards, put them all onto the battlefield,
    /// shuffle the rest in" (Synthetic Destiny); stops on an empty library.
    /// `rest_bottom` bottoms the rest at random instead (Empty the Laboratory).
    RevealUntilMatchingToBattlefield {
        filter: SelectionRequirement,
        count: Value,
        #[serde(default)]
        rest_bottom: bool,
    },
    /// Lukka, Coppercoat Outcast's −2:
    /// [`Self::RevealUntilOneToBattlefieldRestBottom`] over creatures with
    /// greater mana value than `Selector::LastMoved` (the creature it just
    /// exiled).
    RevealUntilCreatureAboveLastMovedToBattlefield,
    /// "Reveal until a [filter] card; put it onto the battlefield, the rest on
    /// the bottom at random." `damage_controller`: damage to you per card
    /// revealed (Audacious Reshapers). `may`: putting it in is optional, else it
    /// is bottomed too (Genesis Storm).
    RevealUntilOneToBattlefieldRestBottom {
        filter: SelectionRequirement,
        damage_controller: bool,
        #[serde(default)]
        may: bool,
    },
    /// "Until your next turn, whenever a creature attacks you or a
    /// planeswalker you control, [body]" — registers a floating trigger;
    /// the attacker is bound as `Selector::TriggerSource`. Tamiyo +2.
    OnAttackedUntilYourNextTurn { body: Box<Effect> },
    /// "Until end of turn, whenever a `filter` creature attacks, [body]", any
    /// player's attacker, bound as `TriggerSource` (Summon: Leviathan).
    OnMatchingAttacksThisTurn { filter: SelectionRequirement, body: Box<Effect> },
    /// "Until end of turn, whenever a [filter] creature blocks, [body]" — the
    /// block-side `OnMatchingAttacksThisTurn` (Benefactor's Draught). `filter`
    /// reads from the registering controller; the blocker is `TriggerSource`.
    OnMatchingBlocksThisTurn { filter: SelectionRequirement, body: Box<Effect> },
    /// "Whenever a creature blocks this turn, its controller gets `amount`
    /// poison counters" (Noxious Assault). Turn-scoped flag consumed at
    /// blocker declaration.
    BlockersPoisonedThisTurn { amount: u32 },
    /// Stagger (Lightning, Army of One) — until your next turn, if a source
    /// would deal damage to `who` or a permanent they control, it deals
    /// double that damage instead (CR 614.5-style replacement).
    StaggerPlayerUntilYourNextTurn { who: PlayerRef },
    /// Sacrifice the `filter` permanent with the greatest mana value (Soul
    /// Shatter), or greatest power with `by_power` (Crackling Doom). Auto picks
    /// the top one per player.
    SacrificeGreatestMV {
        who: Selector,
        count: Value,
        filter: SelectionRequirement,
        #[serde(default)]
        by_power: bool,
    },

    /// Punisher ("unless"): each `chooser` player may avoid `otherwise` by
    /// doing one of `options`. Heuristic: the first affordable option (life
    /// within their total, a legal sacrifice); if none, `otherwise` runs for the
    /// controller. Options run with the chooser as controller (use
    /// `PlayerRef::You`). Indulgent Tormentor.
    Punisher {
        chooser: Selector,
        options: Vec<Effect>,
        otherwise: Box<Effect>,
    },

    /// CR 701.55 — villainous choice: each `who` (APNAP) picks an option, run
    /// with them as controller (`You` = chooser). No fallback payoff, unlike
    /// `Punisher`. The bot picks the lower self-harm option; UI prompting is a
    /// follow-up (TODO.md).
    VillainousChoice {
        who: Selector,
        option_a: Box<Effect>,
        option_b: Box<Effect>,
    },

    // ── Counters on players ──────────────────────────────────────────────────
    AddPoison { who: Selector, amount: Value },
    /// CR 122.1i / 729 — give each resolved player `amount` rad counters.
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

    /// Reveal `who`'s top card; `then` if it matches `filter`, else `else_` if
    /// any. It stays on top. CHK Deceivers, Paroxysm.
    RevealTopThenIf {
        who: PlayerRef,
        filter: SelectionRequirement,
        then: Box<Effect>,
        #[serde(default)]
        else_: Option<Box<Effect>>,
    },

    /// Reveal the top card of `who`'s library; if it's a permanent card, put it
    /// onto the battlefield under its owner's control (firing ETB). Otherwise
    /// it stays on top. Chaos Warp.
    RevealTopPutPermanentOntoBattlefield { who: PlayerRef },
    /// Arbiter of the Ideal: reveal `who`'s top card; a `filter` match may enter
    /// with a `counter` counter and `extra_types`; otherwise it stays on top.
    RevealTopMayPutOntoBattlefield {
        who: PlayerRef,
        filter: SelectionRequirement,
        #[serde(default)]
        counter: Option<crate::card::CounterType>,
        #[serde(default)]
        extra_types: Vec<crate::card::CardType>,
    },
    /// Gishath, Sun's Avatar: reveal the top `count`; every `filter` card enters
    /// under `who`, the rest are bottomed ("any number" = all).
    RevealTopNPutMatchingToBattlefield { who: PlayerRef, count: Value, filter: SelectionRequirement },

    /// Reveal `who`'s top card: a land enters (`tapped`: Risen Reef), otherwise
    /// to hand — or with `nonland_stays` it stays on top (Explorer's Scope).
    /// Coiling Oracle. `may`: the land may stay off the battlefield (then hand,
    /// or top with `nonland_stays`).
    RevealTopLandToBattlefieldElseHand {
        who: PlayerRef,
        #[serde(default)]
        tapped: bool,
        #[serde(default)]
        nonland_stays: bool,
        #[serde(default)]
        may: bool,
    },

    /// Look at the top card of `who`'s library; if it's a land, put it into
    /// their hand, otherwise they may mill it (a UI seat is asked; headless
    /// seats bin it). Traveling Botanist, Sarinth Steelseeker.
    LookTopLandToHandElseBin { who: PlayerRef },

    /// Reveal the top card of `who`'s library; if it's a permanent card with
    /// mana value ≤ `max_mv`, put it onto the battlefield; otherwise put it
    /// into their hand. Matter Reshaper's death trigger.
    RevealTopPutPermanentMvElseHand { who: PlayerRef, max_mv: Value },

    /// "You may put a [filter] card you own from your hand or the command zone
    /// onto the battlefield. If you do, [then]" (Next of Kin). Not a cast: a
    /// commander pays no tax (CR 903.8).
    PutFromHandOrCommandZoneOntoBattlefield {
        filter: SelectionRequirement,
        then: Option<Box<Effect>>,
    },
    /// "You may put a [filter] card from your hand onto the battlefield": up to
    /// `count` from `who`'s hand via `ChooseCards` (always optional), under the
    /// controller, `tapped` if set (Elvish Piper). `haste` and `sacrifice_eot`
    /// make Sneak Attack.
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

    /// Phyrexian Splicer: pick one of `options`; `from` loses it and `to` gains
    /// it this turn. No-op if `from` lacks it.
    MoveChosenKeyword { options: Vec<Keyword>, from: Selector, to: Selector },

    /// CR 123.3 / 123.6 — "(you may) put a name sticker on `what`": one of the
    /// controller's unused stickers, ballot ordered most unique vowels first (so
    /// a first-option bot takes the richest); `optional` adds "no sticker". Not
    /// on an object they don't own (CR 123.3b). Recorded for
    /// [`Value::NameStickerUniqueVowels`] (0 when none).
    PutNameSticker { what: Selector, optional: bool },

    /// Scroll Rack — "Exile any number of cards from your hand face down. Put
    /// that many cards from the top of your library into your hand. Then look
    /// at the exiled cards and put them on top of your library in any order."
    ScrollRack,

    /// Echo Chamber — `who` chooses a creature they control; its controller's
    /// opponent (the effect's controller) gets a hasty token copy that is
    /// exiled at the beginning of the next end step.
    TokenCopyOfOpponentChoice { who: PlayerRef },

    /// No Quarter: on a block trigger, destroy the weaker of the pair —
    /// `attacker_side: false` the outclassed blocker, `true` the outclassed
    /// attacker.
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
    /// CR 702.62a — Rousing Refrain: the resolving spell is exiled with its
    /// printed suspend N time counters instead of the graveyard. A copy just
    /// ceases to exist (CR 707.10a).
    ExileSelfSuspended,

    /// CR 702.18 — "You gain shroud until end of turn" (Gilded Light). No
    /// player, `who` included, may target them for the rest of the turn.
    PlayerGainsShroudThisTurn { who: PlayerRef },
    /// CR 702.16 — "you gain protection from the color of your choice until
    /// end of turn" (Seht's Tiger). The pick is the hostile color.
    PlayerGainsProtectionFromChosenColor { who: PlayerRef },
    /// "Each opponent can't block with more than `n` creatures this combat"
    /// (Mirri, Weatherlight Duelist). Sets each opponent's
    /// `block_cap_this_combat`.
    OpponentsBlockWithAtMost { n: u8 },

    /// Dimensional Breach — exile all permanents; for as long as any remain
    /// exiled this way, each player returns one they own to the battlefield
    /// at the beginning of their upkeep.
    DimensionalBreach,

    /// The per-upkeep half of [`Effect::DimensionalBreach`]: the active player
    /// returns one of the cards exiled by the source that they own.
    DimensionalBreachReturn,

    /// Day of the Dragons — exile all creatures you control, then create that
    /// many `token` tokens. The exiled cards come back when the source leaves.
    ExileYourCreaturesForDragons { token: std::sync::Arc<crate::card::TokenDefinition> },

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

    /// Marshland Bloodcaster: `who`'s next spell this turn may be paid in life
    /// equal to its mana value (`Player::life_alt_next_spell_this_turn`, spent
    /// by that spell either way).
    NextSpellThisTurnMayCostLife { who: PlayerRef },
    /// "Return it to the battlefield face down under its owner's control.
    /// It's a Forest land" (Yedora, Grave Gardener) — every card `what`
    /// resolves to in a graveyard enters as a nameless face-down Forest.
    ReturnFaceDownAsForest { what: Selector },
    /// Put `what` onto the battlefield face down as a 2/2 Cyberman artifact
    /// creature (Missy), from any zone; a permanent turns face down in place
    /// (Cyber Conversion).
    PutFaceDownAsCyberman { what: Selector, tapped: bool },
    /// Revival Experiment (CR 110.4): return up to one card per permanent type
    /// (one type per card, priciest), losing `life_per_card` each.
    ReturnOnePerPermanentType { life_per_card: i32 },

    /// Fortunate Few: from the controller in turn order, each seat spares one
    /// unchosen nonland permanent it doesn't control (the cheapest); all other
    /// nonland permanents are destroyed.
    EachPlayerSparesOneTheyDontControl,

    /// Ends an "until your next turn" `GrantActivatedAbilityToMatching` with
    /// `Duration::Permanent` by stripping `ability`; scheduled by a `DelayUntil`
    /// (Song of Freyalise).
    RevokeGrantedActivatedAbility {
        filter: SelectionRequirement,
        ability: Box<crate::effect::ActivatedAbility>,
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
    ExileTopFaceDownTokenReturns { token: std::sync::Arc<crate::card::TokenDefinition> },

    /// Ilharg, the Raze-Boar: may put a creature from hand onto the battlefield
    /// tapped and attacking (CR 508.4 — not declared, no attack triggers; its
    /// controller picks the defender), returned to hand at the next end step.
    DeployCreatureFromHandAttacking {
        filter: SelectionRequirement,
        /// `true` returns the creature to hand at the next end step; `false`
        /// leaves it (a permanent deploy).
        return_to_hand_eot: bool,
        /// "… attacking that opponent" (Kaalia of the Vast): it attacks the
        /// source's defender, no choice.
        #[serde(default)]
        same_defender: bool,
    },

    /// "Put up to `count` land cards from your hand and/or graveyard onto the
    /// battlefield tapped." Deploys as many lands as available (up to `count`),
    /// preferring the graveyard so hand lands stay playable. Worldsoul's Rage.
    DeployLandsFromHandAndGraveyard { count: Value },
    /// "Put up to `count` land cards from your hand onto the battlefield
    /// tapped" (The Gitrog, Ravenous Ride).
    PutLandsFromHandOntoBattlefieldTapped { count: Value },

    /// CR 701.40 — Manifest: put the top `amount` cards of `who`'s library
    /// onto the battlefield face down as 2/2 creatures (the real card is
    /// stashed and can be turned face up for its mana cost if it's a creature).
    Manifest { who: PlayerRef, amount: Value },
    /// CR 701.13 — "Exile all [filter] cards from [who]'s graveyard in a
    /// face-down pile, shuffle that pile, then manifest those cards" under
    /// the resolving controller (Ghastly Conscription).
    ManifestFromGraveyard { who: PlayerRef, filter: SelectionRequirement },
    /// Ashcloud Phoenix: each graveyard card enters face down as a 2/2 under you,
    /// like a manifest (a morph can be turned up).
    PutFaceDownOntoBattlefield { what: Selector },
    /// Magar of the Magic Strings: each `what` graveyard card enters face down
    /// under the controller as a 3/3 ([`facedown_noted_spell_definition`]) that
    /// runs `on_hit` on combat damage to a player and is exiled if it would
    /// leave. `Selector::This` in `on_hit` is the puppet; `CopyCardAndCastFree`
    /// copies its face-up card.
    ///
    /// [`facedown_noted_spell_definition`]: crate::card::facedown_noted_spell_definition
    PutFaceDownAsNotedSpell { what: Selector, on_hit: Box<Effect> },
    /// CR 701.34 — the player manifests `count` cards from hand;
    /// `controller_draws`: you draw one per manifest (Kozilek, the Broken
    /// Reality).
    ManifestFromHand { who: Selector, count: Value, controller_draws: bool },
    /// Create a token and attach it to the resolved permanent (Role tokens
    /// — Wicked Role; CR 111.10). The token must be an Aura-style
    /// attachment; it enters attached.
    CreateTokenAttachedTo { target: Selector, definition: std::sync::Arc<crate::card::TokenDefinition> },
    /// `CreateTokenAttachedTo` once per `target` permanent (CR 111.10) — Asinine
    /// Antics.
    CreateTokenAttachedToEach { target: Selector, definition: std::sync::Arc<crate::card::TokenDefinition> },
    /// Indomitable Creativity: destroy up to X `filter` targets (slots `0..X`);
    /// for each, its controller reveals until an artifact or creature card, puts
    /// it onto the battlefield and shuffles the rest in.
    DestroyTargetsPolymorph { filter: SelectionRequirement },
    /// Destroy X chosen permanent targets matching `filter` (slots `0..X`
    /// from the cast's target list) — the plain sibling of
    /// `DestroyTargetsPolymorph`. Heliod's Intervention mode 0.
    DestroyTargets { filter: SelectionRequirement },
    /// Sundering Titan: destroy one land per basic type (auto-picked, preferring
    /// opponents').
    DestroyLandOfEachBasicType,
    /// CR 702.72 — champion a [filter]: exile another of yours linked to the
    /// source, or sacrifice the source if none (Mistbind Clique).
    Champion { filter: SelectionRequirement },
    /// Exile up to `count` graveyard cards chosen by the controller (Faerie
    /// Macabre). `of` limits to one player's graveyard (Skullsnatcher); `single`
    /// to one graveyard, set by the first pick (Rag Dealer). UI picks via
    /// `ChooseCards`; auto takes the priciest opponent cards.
    ExileUpToNFromGraveyards { count: Value, of: Option<PlayerRef>, single: bool },
    /// Choose a color, exile the top `amount` cards of `who`'s library, and
    /// create one `token` per exiled card of the chosen color (Oona, Queen
    /// of the Fae).
    ExileTopMintPerChosenColor { who: Selector, amount: Value, token: std::sync::Arc<crate::card::TokenDefinition> },
    /// Elspeth Conquers Death II: opponents' `filter` spells cost `{amount}` more
    /// until the controller's next untap.
    SpellTaxUntilYourNextTurn { amount: u32, filter: SelectionRequirement },
    /// Tragic Arrogance: `SacrificeAllButOnePerType` (nonland) with the
    /// resolving controller choosing for every player.
    SacrificeAllButOnePerTypeYouChoose { who: Selector },
    /// Cataclysm family: each resolved player keeps one artifact, creature,
    /// enchantment and planeswalker among their nonland permanents and
    /// sacrifices the rest (Ajani, Nacatl Avenger's -4; `effects/keep_one.rs`).
    SacrificeAllButOnePerType {
        who: Selector,
        /// Cataclysm also lets each player keep one land ("an artifact, a
        /// creature, an enchantment, and a land"). `false` is the
        /// nonland-only Ajani wording.
        #[serde(default)]
        include_land: bool,
    },
    /// "Each player chooses a [`filter`] permanent they control, then sacrifices
    /// the rest" (Deadly Vanity) — `SacrificeAllButOnePerType` for one filter,
    /// keepers picked APNAP (bot: priciest). `destroy`: destroys instead (Divine
    /// Reckoning; CR 701.8).
    EachPlayerKeepsOneSacrificeRest {
        who: Selector,
        filter: SelectionRequirement,
        #[serde(default)]
        destroy: bool,
        /// "Return the rest to their owners' hands" instead (Consuming Tide).
        #[serde(default)]
        bounce: bool,
    },
    /// The Legend of Yangchen: from the controller in turn order (CR 101.4), each
    /// seat picks up to one [filter] permanent your opponents control; all are
    /// exiled at once (CR 608.2c). Headless names the priciest it doesn't
    /// control.
    EachPlayerChoosesOpponentPermanentExile { filter: SelectionRequirement },
    /// Wish: a `filter` card you own from your sideboard to hand (Burning Wish);
    /// `Decision::ChooseCards` for UI, first match otherwise. Commander has no
    /// sideboard, so it finds nothing.
    WishToHand { filter: SelectionRequirement },
    /// Extrapolate the Impossible: reveal two differently named sideboard cards;
    /// an opponent (the most hostile, or the caster's pick at 3+ seats) picks one
    /// for your hand. Fewer than two names: nothing happens.
    WishPairOpponentChooses,
    /// `WishToHand`, "or choose a face-up [filter] card you own in exile"
    /// (Karn, the Great Creator's −2) — only that card reaches into exile.
    WishToHandOrFaceUpExile { filter: SelectionRequirement },
    /// "Shuffle up to `max` cards you own matching `filter` from outside the
    /// game into your library" (Research // Development's left half).
    WishToLibrary { filter: SelectionRequirement, max: Value },
    /// Unyielding Gatekeeper: exile the target; if you controlled it, it returns
    /// tapped, else its controller does `theirs` (control read as it left).
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

    /// Valgavoth's Onslaught: manifest dread `count` times, then `counters`
    /// +1/+1 on each; self-contained so the set doesn't leak into
    /// `last_moved_cards`.
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

    /// CR 702.182 — Become Anonymous: exile `what` and the top `count` face down,
    /// shuffle, cloak them tapped. Only what reached exile joins (a commander
    /// sent home doesn't).
    CloakPileWithTop { what: Selector, count: Value },

    /// Manifold Insights: reveal the top `count`; from the next opponent in turn
    /// order, each opponent picks a different `pick_filter` card for your hand;
    /// the rest bottom at random. Bots hand over the cheapest left.
    RevealTopEachOpponentChoosesToHand { count: Value, pick_filter: SelectionRequirement },
    /// Karn, Scion of Urza's +1: reveal the top `count`; an opponent picks one
    /// for your hand, the rest are exiled (with `counter` if `Some`). UI
    /// opponents are prompted; bots hand over the cheapest.
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

    /// "Return a [filter] card of an opponent's choice from your graveyard to
    /// your hand", untargeted (Tasigur). Bots hand back the cheapest. `chooser`
    /// names that opponent (O-Kagachi Made Manifest); `None` lets the controller
    /// pick. On `LastMoved`.
    ReturnFromGraveyardOpponentChooses {
        filter: SelectionRequirement,
        #[serde(default)]
        chooser: Option<PlayerRef>,
    },

    /// Menacing Ogre: everyone secretly picks up to `max`; the highest loses that
    /// much life; `on_you_win` runs if the controller is among them. Via
    /// `ask_seat_amount`, so UI seats are prompted through suspend.
    EachPlayerChoosesNumberHighestLoses { max: u32, on_you_win: Box<Effect> },
    /// Wheel of Misfortune: secret numbers 0..=`max`; the highest is dealt that
    /// much by the source, everyone not lowest wheels for seven. Headless picks
    /// 0, its controller 1.
    WheelOfMisfortune { max: u32 },

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

    /// Gisa, Glorious Resurrector: every creature card exiled by this source
    /// enters under you, optionally with `Keyword::Decayed`.
    ReturnExiledBySourceToBattlefield {
        #[serde(default)]
        decayed: bool,
        /// "Return **a** card exiled with this enchantment" (Purgatory) — cap
        /// the return at `count` cards instead of emptying the pile.
        #[serde(default)]
        count: Option<Value>,
    },

    /// Kianne, Dean of Substance — exile the top card of the controller's
    /// library; if it's a land card, put it into their hand, otherwise leave it
    /// exiled with one `counter` counter on it.
    StudyTopCard { counter: crate::card::CounterType },

    /// Imbraham, Dean of Theory — exile the top `count` cards of the
    /// controller's library, each with one `counter` counter on it.
    ExileTopWithCounters { count: Value, counter: crate::card::CounterType },

    /// CR 401.6 — this turn you may look at and play from the top of your
    /// library (`Player.play_from_top_this_turn`). The Belligerent.
    GrantPlayFromTopThisTurn,
    /// `GrantPlayFromTopThisTurn` whose spells cost life equal to their mana
    /// value rather than their mana cost — Bolas's Citadel for a turn
    /// (Gwenom, Remorseless).
    GrantPlayFromTopPayLifeThisTurn,
    /// "Until end of turn, you may look at and play the top card of [who]'s
    /// library, mana as any color" (Xanathar, Guild Kingpin): adds the seat to
    /// `Player.may_play_library_top_of`.
    MayPlayLibraryTopOfThisTurn { who: PlayerRef },
    /// Sen Triplets: `who` plays with their hand revealed and you may play from
    /// it this turn (`Player.may_play_hand_of`, `GameState::hand_visible_to`).
    MayPlayHandOfThisTurn { who: PlayerRef },

    /// Ashiok, Nightmare Muse −7: free end-of-turn `may_play_until` on up to
    /// `count` face-up opponent-owned exiled cards. The spell cap is just the
    /// number of cards granted.
    CastUpToNFromOpponentsExile { count: Value },

    /// Omnath, Locus of Creation: `branches[n]` for this ability's n-th
    /// resolution this turn; past the last, nothing (`EscalatingThisTurn`
    /// repeats the last).
    NthResolutionThisTurn { branches: Vec<Effect> },

    /// Scholarship Sponsor — each player controlling fewer lands than the
    /// player with the most searches their library for up to (difference)
    /// basic land cards, puts them onto the battlefield tapped, then shuffles.
    CatchUpBasicLands {
        /// The land count each player is brought up to. `None` = the current
        /// leader's total (Scholarship Sponsor); `Some(n)` a flat bar
        /// (Natural Balance's five).
        #[serde(default)]
        target: Option<Value>,
        /// The fetched basics enter tapped (Scholarship Sponsor).
        #[serde(default)]
        tapped: bool,
    },

    /// Uvilda, Dean of Perfection: may exile an instant / sorcery from hand with
    /// `count` hone counters (`GameState::process_hone` does the rest).
    HoneFromHand { count: Value },

    /// Controller chooses `count` cards from their hand and puts them on top of
    /// their library in a chosen order (first chosen = topmost).
    PutOnLibraryFromHand { who: PlayerRef, count: Value },

    /// Each player in `who` puts one card from their hand on top of their
    /// library (Sadistic Augermage). Suspends per-seat for UI players via the
    /// same continuation machinery as a symmetric discard.
    EachPlayerPutsHandCardOnTop { who: Selector },

    /// Sacrifice one `filter` creature of `who`'s, storing its power for
    /// `Value::SacrificedPower` (Thud).
    SacrificeAndRemember { who: PlayerRef, filter: SelectionRequirement },

    /// Destroy and record P/T / mana value like [`Effect::SacrificeAndRemember`],
    /// for a following `Value::SacrificedToughness` (Orzhov Charm).
    DestroyAndRemember { what: Selector },

    /// Internal: re-stamp the P/T and mana value of a creature sacrificed as an
    /// activation cost before `body`, so `Value::SacrificedPower/Toughness` and
    /// `ManaValueEqualsSacrificedPlus` survive intervening resolutions (Witch's
    /// Oven). Wrapped by `activate_ability`; not for card definitions.
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
        /// Its type / colour flags (`Predicate::SacrificedWas*`,
        /// `SharesColorWithSacrificed`), which a cost paid in response would
        /// otherwise overwrite before this resolves.
        #[serde(default)]
        traits: Option<Box<SacrificedTraits>>,
        body: Box<Effect>,
    },

    /// Internal (CR 701.50a): the end of one connive — a +1/+1 counter on
    /// `conniver` per nonland discard since entry `since`, then `Connived`. Its
    /// own node so a suspended discard parks it.
    ConniveFinish { conniver: crate::card::CardId, controller: usize, since: u32 },

    /// Internal: re-stamp the power tapped for a Station cost (CR 702.184a)
    /// before `body`, for `Value::TappedForCostPower`. Wrapped by
    /// `activate_ability`.
    WithTappedPower { power: i32, body: Box<Effect> },
    /// Internal: re-stamp Treasure mana spent on the activation before `body`,
    /// for `Value::TreasureManaSpentToActivate`. Wrapped by `activate_ability`.
    WithTreasureManaSpent { amount: u32, body: Box<Effect> },
    /// Internal: re-stamp the counters `remove_all_counters_cost` removed before
    /// `body`, for `Value::CountersRemovedAsCost` (paid at activation, CR
    /// 602.2b). Wrapped by `activate_ability`; not for card definitions.
    WithCountersRemovedAsCost { amount: u32, body: Box<Effect> },
    /// Internal: re-stamp what the activation's costs exiled, tapped, sacrificed
    /// and discarded before `body` (paid at activation, CR 602.2b); a response
    /// activation used to erase them (Tawnos, Coin of Fate). Wrapped by
    /// `activate_ability` when any is set; not for card definitions.
    WithPaidCosts { paid: Box<PaidCostRecord>, body: Box<Effect> },

    /// Internal: re-stamp nonland cards discarded for an additional cost (CR
    /// 601.2h) before `body`, for `Predicate::DiscardedNonlandThisEffect` (Grab
    /// the Prize). Wrapped by the cast path.
    WithCastDiscards { nonland: u32, body: Box<Effect> },

    /// Internal: re-stamp the (power, mana value) revealed for an
    /// `AdditionalCastCost::RevealFromHand` before `body`, so
    /// `Value::RevealedForCost*` reads this spell's reveal despite responses.
    /// Wrapped by the cast path.
    WithRevealedForCost { power: i16, mana_value: i16, body: Box<Effect> },
    /// Internal: the card discarded for this activation's cost, for
    /// `Effect::IfDiscardedForCost`. Wrapped by `activate_ability` only when
    /// needed.
    WithDiscardedForCost { card: crate::card::CardId, body: Box<Effect> },
    /// "If the discarded card was a [filter] card, [then]" — the card
    /// discarded to pay this activation's cost (Moria Scavenger's "if the
    /// discarded card was a creature card, amass Orcs 1").
    IfDiscardedForCost { filter: SelectionRequirement, then: Box<Effect> },

    /// CR 406.3 — "exile … face down": run `body`, then turn every card it
    /// put into exile face down. Only a may-play holder (else the controller)
    /// may look at one (Intet, Kheru Mind-Eater, Predators' Hour).
    ExileFaceDown { body: Box<Effect> },

    /// CR 701.16a — investigate: `body` is the Clue `CreateToken`; each Clue its
    /// `count` asks for is one investigation per `who` player
    /// (`GameEvent::Investigated`), whatever replacements make. Built by
    /// `shortcut::investigate*`.
    Investigate { body: Box<Effect> },

    /// "Target opponent reveals their hand; you choose a [filter] card; they
    /// discard it" (Thoughtseize). The caster auto-picks the first match; a
    /// picker UI is future work.
    DiscardChosen {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
    },
    /// [`Effect::DiscardChosen`] over only `reveal` revealed cards (Disciple of
    /// Phenax), taken in hand order; the caster picks `pick` (Noggin Whack).
    DiscardChosenFromRevealed {
        from: Selector,
        reveal: Value,
        #[serde(default = "crate::effect::one_value")]
        pick: Value,
    },
    /// Vendilion Clique: look at `from`'s hand, choose `count` `filter` cards;
    /// they bottom them and draw that many.
    ///
    /// Approximations: the "may" is forced-if-able (first match, like
    /// `DiscardChosen`); `from` follows the ETB `EachOpponent` convention
    /// (trigger player-targeting is an engine-wide gap, see Archon of Cruelty),
    /// so the "target player" self-cast mode isn't available.
    BottomChosenFromHandAndDraw {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
    },
    /// Painful Memories: choose `count` `filter` cards from `from`'s hand to go
    /// on top — [`Effect::BottomChosenFromHandAndDraw`] on top, no draw.
    TopChosenFromHand {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
    },
    /// Elite Spellbinder: exile a `filter` card from `from`'s hand; its owner may
    /// play it for `extra_cost` more. Approximated as a may-play through their
    /// next turn rather than "as long as it remains exiled".
    ExileFromHandTaxed {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
        extra_cost: u32,
    },
    /// CR 603.6e — "target player reveals their hand; you choose [count]
    /// [filter] cards; exile them until this leaves", linked to the source
    /// (Tidehollow Sculler, `return_to` = Hand).
    ExileChosenUntilSourceLeaves {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
        return_to: crate::card::ExileReturnZone,
    },

    /// "Reveal your hand; I choose [count] [filter] cards to exile" for good,
    /// unlike `ExileChosenUntilSourceLeaves` (Thought-Knot Seer).
    /// `link_to_source` stamps `exiled_with = source` (Bane Alley Broker);
    /// `face_down` hides them.
    ExileChosenFromHand {
        from: Selector,
        count: Value,
        filter: SelectionRequirement,
        #[serde(default)]
        link_to_source: bool,
        #[serde(default)]
        face_down: bool,
    },

    /// `what` gains a card type in addition to its others, indefinitely (ends
    /// when it leaves) — Phyrexian Scriptures I. `until_eot`: a CR 611.2
    /// end-of-turn effect (Liquimetal Torque).
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

    /// Grant `what` an activated ability. `Duration::Permanent` anchors it to
    /// the permanent (`granted_activated_abilities`, gone when it leaves — Urza's
    /// Saga); `EndOfTurn` / `EndOfCombat` use `granted_activated_eot`, cleared at
    /// cleanup.
    GainActivatedAbility { what: Selector, ability: Box<ActivatedAbility>, duration: Duration },

    // ── Delayed triggers and pact costs ──────────────────────────────────────
    /// Register a delayed trigger: `kind` is the future event, `body` the
    /// effect. Captures `ctx.targets[0]` for `Selector::Target(0)`.
    DelayUntil {
        kind: DelayedTriggerKind,
        body: Box<Effect>,
    },
    /// `DelayUntil` with `capture` resolved now as the body's slot 0, so "that
    /// creature" survives the wait (Gift of Immortality).
    DelayUntilWithCapture {
        kind: DelayedTriggerKind,
        capture: Selector,
        body: Box<Effect>,
    },
    /// "Add {C} equal to [amount] at the beginning of your next main phase"
    /// (Mana Drain): `amount` is read now, while scratch like
    /// `CounteredSpellManaSpent` is live, and baked into the delayed trigger.
    AddManaAtNextMainPhase {
        amount: Value,
        /// When true the mana is added as `AnyColors` (the player picks each
        /// pip) rather than {C} — Plasm Capture's "in any combination of
        /// colors". Defaults to false (Mana Sculpt's colorless bank).
        #[serde(default)]
        any_color: bool,
        /// "Your next *first* main phase" (Plasm Capture): only a precombat
        /// main counts. False is "your next main phase", first or second
        /// (CR 505.1).
        #[serde(default)]
        first_main: bool,
    },

    /// "When this creature dies this turn, [body]" (Golden Guardian):
    /// [`Self::WhenTargetDiesThisTurn`] on the source, the body run from its card
    /// (`ExileSelfReturnTransformed` lifts it out of the graveyard).
    WhenSourceDiesThisTurn { body: Box<Effect> },
    /// "When [target creature] dies this turn, [body]": a delayed trigger on
    /// `ctx.targets[slot]`'s death, its controller captured so
    /// `Selector::Target(0)` survives the death. Expires at cleanup. Searing
    /// Blood (slot 0), Devouring Tendrils (slot 1).
    WhenTargetDiesThisTurn {
        body: Box<Effect>,
        /// Which target slot to watch (default 0 via `#[serde(default)]`).
        #[serde(default)]
        slot: usize,
        /// Cast-time filter for the watched target when this is the slot's only
        /// mention (Melira); `None` when an earlier effect declared it (Searing
        /// Blood).
        #[serde(default)]
        filter: Option<SelectionRequirement>,
    },

    /// "Whenever [target creature] attacks one of your opponents this turn,
    /// [body]" (Maeve, Insidious Singer): watches `ctx.targets[slot]`.
    WhenTargetAttacksOpponentThisTurn { body: Box<Effect>, slot: usize },
    /// "This turn, when [the creature in target slot `slot`] attacks and
    /// isn't blocked, [body]" (Delif's Cone): fires once, on its
    /// `AttackerWentUnblocked`; the creature is `Selector::TriggerSource`.
    WhenTargetAttacksUnblockedThisTurn { body: Box<Effect>, slot: usize },
    /// The untimed `WhenTargetDiesThisTurn` (Xira, the Golden Sting): outlives
    /// cleanup; the dead creature is the trigger source. `if_had_counter` reads
    /// the death's last-known counters.
    WhenTargetDies { body: Box<Effect>, slot: usize, if_had_counter: Option<crate::card::CounterType> },

    /// CR 603.4 — "whenever a creature you control enters this turn, [body]",
    /// once per entrant (`Selector::TriggerSource`); expires at cleanup (First
    /// Day of Class).
    CreaturesYouControlEnteringThisTurn {
        body: Box<Effect>,
    },

    /// Crypt Champion: each player reanimates their priciest creature with mana
    /// value ≤ `max_mv`, APNAP. UI players don't choose (approximation).
    EachPlayerReanimateCreatureMaxMv {
        max_mv: u32,
    },

    /// CR 603.4 — "whenever target creature deals damage this turn, you gain that
    /// much life" (Paladin of Prahv); slot `slot`, amount via
    /// `TriggerEventAmount`; expires at cleanup.
    GainLifeWhenTargetDealsDamageThisTurn {
        #[serde(default)]
        slot: usize,
    },

    /// CR 603.4 — "whenever [slot `slot`] deals combat damage to a player this
    /// turn, [body]" (Captain Howler); amount via `TriggerEventAmount`.
    /// `this_combat`: only this combat phase (Stensian Sanguinist; CR 506.1).
    WhenTargetDealsCombatDamageToPlayerThisTurn {
        #[serde(default)]
        slot: usize,
        body: Box<Effect>,
        #[serde(default)]
        this_combat: bool,
    },
    /// CR 603.7 — the untargeted `WhenTargetDealsCombatDamageToPlayerThisTurn`
    /// for `what`, firing once (Optimus Prime, Autobot Leader, under a
    /// `ForEach`).
    WhenObjectDealsCombatDamageToPlayerThisTurn { what: Selector, body: Box<Effect> },
    /// Jeweled Amulet: stamp the source's `chosen_color` with the color spent on
    /// the activation (cleared for colorless), for
    /// `ManaPayload::ChosenColorOfSource`.
    NoteSpentManaTypeOnSource,

    /// CR 603.4 — "until end of turn, whenever a creature you control dies,
    /// [body]" (Waltz of Rage), read from death LKI; the dead creature is
    /// `TriggerSource`.
    CreaturesYouControlDyingThisTurn {
        body: Box<Effect>,
    },

    /// CR 603.4 — until end of turn, `body` per dying creature (anyone's) whose
    /// LKI matches `filter`; dead one = `TriggerSource`. Massacre Girl.
    WheneverCreatureDiesThisTurn {
        filter: SelectionRequirement,
        body: Box<Effect>,
    },

    /// CR 603.4 — this turn, `body` per entering `filter` creature (anyone's) as
    /// `TriggerSource` (Theoretical Duplication).
    WheneverCreatureEntersThisTurn {
        filter: SelectionRequirement,
        body: Box<Effect>,
    },

    /// Until your next turn, `body` per entering `filter` creature
    /// (`TriggerSource`) — Kardur, Doomscourge (2021 ruling).
    WheneverCreatureEntersUntilYourNextTurn {
        filter: SelectionRequirement,
        body: Box<Effect>,
    },

    /// CR 603.4 — "until end of turn, whenever a creature you control deals
    /// combat damage to a player, [body]" (Mistway Spy); the dealer is
    /// `TriggerSource`.
    CreaturesYouControlDealingCombatDamageThisTurn {
        body: Box<Effect>,
    },
    /// "Until end of turn, whenever one or more [filter] creatures you control
    /// deal combat damage to a player, [body]" — once a batch (CR 603.2c) per
    /// damaged player; that seat is `PlayerRef::TriggerEventPlayer`.
    MatchingCreaturesDealingCombatDamageBatchThisTurn {
        filter: SelectionRequirement,
        body: Box<Effect>,
    },
    /// The Clone Saga III: name a card (bot: its own strongest creature); this
    /// turn, each creature with that name dealing combat damage to a player
    /// fires `body` (CR 603.7), whoever controls it; dealer = `TriggerSource`.
    NameCardThenNamedCreatureCombatDamageThisTurn {
        body: Box<Effect>,
    },

    /// CR 603.4 — "until end of turn, whenever you gain life, [body]" (Vizkopa
    /// Guildmage), amount via `TriggerEventAmount`.
    WheneverYouGainLifeThisTurn { body: Box<Effect> },
    /// "For each 1 damage prevented this way, [body]" after a this-turn fog
    /// (Inkshield): CR 603.4 trigger per prevented combat damage event to you,
    /// amount via `TriggerEventAmount`.
    WheneverCombatDamageToYouPreventedThisTurn { body: Box<Effect> },

    /// CR 603.4 — Pure Intentions: this turn, `body` per discard an opponent's
    /// spell or ability makes you do (the card as trigger source).
    WheneverOpponentMakesYouDiscardThisTurn { body: Box<Effect> },

    /// CR 603.4 — Duskmantle Guildmage: this turn, `body` per card put into an
    /// opponent's graveyard, its owner as `Target(0)`.
    WheneverCardEntersOpponentGraveyardThisTurn { body: Box<Effect> },

    /// Repeating `OnYourNextSpellCastThisTurn` until cleanup (Rediscover the Way
    /// III); gate type filters with `Effect::If` on `TriggerSource`.
    OnEachSpellCastThisTurn { body: Box<Effect> },
    /// CR 603.7e — Codie: `body` on your next spell this turn, one-shot, spell =
    /// `TriggerSource`.
    OnYourNextSpellCastThisTurn {
        body: Box<Effect>,
    },
    /// CR 603.7e — All-Out Assault: `body` on your next attack this turn, after
    /// attackers tap.
    OnYourNextAttackThisTurn {
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
    /// "When you next cast an instant or sorcery spell this turn, copy that
    /// spell X times" (Storm King's Thunder): `count` is evaluated NOW — a
    /// delayed body's `XFromCost` reads the storm count, not this spell's X.
    CopyNextInstantSorceryThisTurn { count: Value },
    /// Game of Chaos, one round at `stake`: flip a coin; the winner gains
    /// `stake` life and the loser loses it, then the winner may play the next
    /// round at twice the stake (asked of them; doubling stops at 2^31).
    GameOfChaosRound { opponent: PlayerRef, stake: u32 },
    /// CR 603.7e — one-shot "when you next cast a `card_type` spell this turn,
    /// [body]" (`TriggerSource` = the spell). Other types leave it armed, so
    /// Smoldering Stagecoach's instant and sorcery riders are two of these.
    OnYourNextSpellOfTypeThisTurn {
        card_type: crate::card::CardType,
        body: Box<Effect>,
    },
    /// CR 603.7e — one-shot on your next `filter` spell this turn, spell =
    /// `TriggerSource`, its X = `Value::XFromCost` (Brass Infiniscope). Others
    /// leave it armed.
    OnYourNextSpellMatchingThisTurn {
        filter: SelectionRequirement,
        body: Box<Effect>,
    },
    /// Medomai's Prophecy III: `OnYourNextSpellCastThisTurn` for the source's
    /// `named_card` only; other casts don't consume it.
    OnYourNextNamedSpellThisTurn {
        body: Box<Effect>,
    },

    /// Signal the Clans: search for three differently named creature cards (the
    /// priciest), take one at random, shuffle. Fewer than three names: nothing
    /// is taken; it always shuffles.
    SignalTheClans,
    /// Unexpected Results: shuffle, reveal the top. Nonland: may cast it free
    /// (`cast_card_for_free`; declined stays on top). Land: may put it onto the
    /// battlefield and return this spell to hand
    /// (`return_resolving_spell_to_hand`).
    UnexpectedResults,
    /// Search library (and graveyard with `graveyard`) for up to `count`
    /// differently named `filter` cards (mana value ≤ X with `mv_at_most_x`); an
    /// opponent sends two to `denied`, the rest to `rest` (Ecological
    /// Appreciation). Auto: priciest in library order; opponent denies the first
    /// two.
    SearchSplitWithOpponent {
        count: u32,
        filter: SelectionRequirement,
        mv_at_most_x: bool,
        graveyard: bool,
        denied: ZoneDest,
        rest: ZoneDest,
        /// The opponent hands over the two SMALLEST (Turtles Forever: the two
        /// it picks go to the caster's hand); `false` denies the two biggest.
        #[serde(default)]
        denied_smallest: bool,
    },
    /// Fact or Fiction: reveal the top `count`; an opponent splits two piles; one
    /// to hand, one to the graveyard (`to_bottom`: the bottom — Jace, Architect
    /// of Thought). Heuristic instead of `Decision::SplitPiles`: the opponent
    /// isolates the priciest card, you keep the higher total.
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

    /// Possibility Storm (trigger source = the spell cast from hand): exile it,
    /// exile from the top until a card sharing a type, may cast that free; all
    /// exiled cards bottom at random.
    PossibilityStorm,

    /// CR 614 — "Permanents enter tapped this turn" (Due Respect). A
    /// turn-scoped blanket replacement over every entry path.
    PermanentsEnterTappedThisTurn,
    /// "Lands you control enter tapped this turn" (Nahiri's Lithoforming) —
    /// the controller-scoped, land-only sibling.
    YourLandsEnterTappedThisTurn,
    /// "Until end of turn, you may cast spells from your hand without paying
    /// their mana costs" (Chandra, Flame's Catalyst): Omniscience for the turn.
    CastHandSpellsFreeThisTurn,
    /// "Until your next turn, creatures your opponents control enter tapped"
    /// (Cone of Cold) — a replacement (CR 614.1c) on the controller's seat.
    OpponentsCreaturesEnterTappedUntilYourNextTurn,
    /// "Until end of turn, if one or more creatures would enter from exile
    /// or after being cast from exile, their owners shuffle them into their
    /// libraries instead" (Don't Blink).
    CreaturesFromExileShuffleThisTurn,
    /// "Put a number of time counters on it equal to its mana value. If it
    /// doesn't have suspend, it gains suspend" (The Parting of the Ways) —
    /// `GrantSuspend` with each card's own mana value as the count.
    GrantSuspendManaValueCounters { what: Selector },
    /// "[who] may exile a [filter] card from their hand with a number of time
    /// counters on it equal to its mana value. If it doesn't have suspend, it
    /// gains suspend" (The Eleventh Doctor, The Wedding of River Song).
    MayExileFromHandSuspended { who: PlayerRef, filter: SelectionRequirement },
    /// Alaundo the Seer: exile a hand card with time counters equal to its mana
    /// value (only the last-counter cast, no upkeep tick, CR 702.62a), then tick
    /// each other exiled card of `who`'s.
    ExileFromHandWithTimeCountersThenTick { who: PlayerRef },
    /// "Choose a suspended card you own and remove `amount` time counters from
    /// it" (Amy Pond) — through the suspend funnel, so the last one casts it
    /// (CR 702.62).
    RemoveTimeCountersFromSuspended { amount: Value },
    /// "Deals `amount` damage to target creature and each other creature that
    /// shares a creature type with it" (Killer).
    DealDamageToTargetAndTypeSharers { what: Selector, amount: Value },
    /// "Exile all creatures except those that share a creature type with a
    /// creature that convoked this spell, all artifacts, and all enchantments"
    /// (Everything Comes to Dust).
    ExileAllButConvokerKin,
    /// "You may cast a spell with suspend from your hand. If you do, pay its
    /// suspend cost rather than its mana cost" (The Face of Boe).
    CastFromHandPayingSuspendCost,

    /// Knowledge Pool's cast replacement — the just-cast spell (the trigger
    /// source) is exiled stamped `exiled_with = source`, and its caster may
    /// then free-cast one of the *other* cards exiled with the source.
    KnowledgePool,

    /// "Pay {cost} or lose the game" (pact upkeeps): auto-pays if affordable,
    /// else the controller loses. No prompt; pacts are ~always paid.
    PayOrLoseGame {
        mana_cost: crate::mana::ManaCost,
        life_cost: u32,
    },

    /// Chancellor of the Annex: `count` first-spell taxes on each player, each
    /// adding {1} to their next spell (`consume_first_spell_tax`).
    AddFirstSpellTax {
        who: PlayerRef,
        count: Value,
    },

    /// Set `Player.sorceries_as_flash` on each resolved player so they may
    /// cast sorcery spells at instant speed until their next turn.
    /// Cleared in `do_untap`. Used by Teferi, Time Raveler's +1.
    GrantSorceriesAsFlash { who: PlayerRef },

    /// Old Stickfingers: reveal until `count` `find` cards, put them into `to`,
    /// bottom the rest at random. One reveal, so it stops at the library's end.
    RevealUntilNFound { who: PlayerRef, find: SelectionRequirement, count: Value, to: ZoneDest },
    /// Vivid (Sanar, Innovative First-Year): reveal until `count` nonland cards;
    /// per color among your permanents, may exile one of that color; shuffle;
    /// cast the exiled cards this turn at cost.
    RevealUntilNonlandsExileOnePerColor { count: Value },
    /// Spoils of the Vault: reveal until a `find` card or `cap` cards; the find
    /// goes to `to`, the rest to `miss_dest`, losing `life_per_revealed` per card.
    /// Auto takes the first match; the printed card-naming is bypassed by
    /// `find`. `miss_dest` defaults to `RevealMissDest::Graveyard` for snapshot
    /// back-compat; SOS cards use `BottomRandom` (Geometer's Arthropod).
    RevealUntilFind {
        who: PlayerRef,
        find: SelectionRequirement,
        to: ZoneDest,
        cap: Value,
        life_per_revealed: u32,
        #[serde(default)]
        miss_dest: RevealMissDest,
    },

    /// Tainted Pact: exile `who`'s top card; take it, or keep digging
    /// (`Decision::OptionalTrigger`, `Bool(true)` = dig), until a card is taken,
    /// a duplicate name is exiled, or the library empties. AutoDecider takes the
    /// first uniquely named card.
    ExileUntilDuplicateName { who: PlayerRef },

    /// `count` extra land plays this turn (`Player.extra_land_plays`) — Explore,
    /// Oracle of Mul Daya.
    GrantExtraLandPlay { who: PlayerRef, count: Value },

    /// "As this enters, choose a creature type" (Cavern of Souls): stored on the
    /// source's `chosen_creature_type`, read by `caster_grants_uncounterable`.
    NameCreatureType { what: Selector },

    /// "As this enters, choose a planeswalker type" (Deification). Asked as a
    /// ballot of the types among cards the controller owns, most-held first;
    /// stamped on `chosen_planeswalker_type`.
    NamePlaneswalkerType { what: Selector },

    /// As [`Effect::NameCreatureType`], but a named player makes the choice
    /// (Callous Oppressor — "an opponent chooses a creature type").
    NameCreatureTypeBy { what: Selector, who: PlayerRef },

    /// [`Effect::NameCreatureType`] over a closed list (A Killer Among Us); an
    /// answer outside `options` is clamped to the first.
    NameCreatureTypeAmong { what: Selector, options: Vec<crate::card::CreatureType> },

    /// CR 201.3 — "as this enters, choose a card name" (Pithing Needle), stored
    /// on `what`'s `named_card`; `activate_ability` suppresses that name's
    /// non-mana abilities.
    ///
    /// CR 201.4a — `restrict_to` narrows to names with matching printed
    /// characteristics ("a land card name" — Petrified Hamlet); off-namespace
    /// answers are rejected. `None` = any name.
    NameCard {
        what: Selector,
        #[serde(default)]
        restrict_to: Option<SelectionRequirement>,
    },
    /// Keep the name `what` just named in its list of chosen names, so
    /// `SelectionRequirement::NameChosenForSource` still matches it after the
    /// next `NameCard` (Day of the Moon's three chapters).
    RememberNamedCard { what: Selector },

    /// Stamp `n` as the source's `chosen_number` — an `AsEntersChooseMode` branch
    /// for statics gated on `Value::ChosenNumberOfSource` (Greymond, Avacyn's
    /// Stalwart).
    SetSourceChosenNumber(u32),
    /// "As [this] enters, choose a letter" (Katerina of Myra's Marvels): the
    /// letter's index (A = 0 … Z = 25) as the source's `chosen_number`, read
    /// by `SelectionRequirement::NameBeginsWithChosenLetterOfSource`.
    ChooseLetterForSource,
    /// "As this enters, choose a number" up to `max`, stored as the source's
    /// `chosen_number` (Sanctum Prelate).
    ChooseNumberForSource {
        max: u32,
        /// The number is life the chooser pays (Phyrexian Processor), so the
        /// ask is a life payment, not "more is better".
        #[serde(default)]
        pays_life: bool,
        /// The lowest number allowed ("between 1 and 10" — Talion).
        #[serde(default)]
        min: u32,
        /// The number is matched against your own creatures' power and
        /// toughness (Squall), so the headless pick is the commonest of those.
        #[serde(default)]
        own_creature_pt: bool,
    },

    /// CR 730 — Shahrazad: a bot-piloted, bounded subgame on the players'
    /// libraries; non-winners lose half their life rounded up (a stalled or
    /// drawn one has no winner).
    PlaySubgame,

    /// CR 509.1 — Sorrow's Path: two blockers swap block assignments if each
    /// could block the other's attackers; else no-op.
    SwapBlockAssignments { a: Selector, b: Selector },

    /// CR 614 — grant Kumano's rider until end of turn: a creature `what`
    /// damages this turn is exiled instead of dying (Runesword).
    GrantDamageExilesVictimThisTurn { what: Selector },

    /// CR 701.19 — "if `what` deals damage to a creature this turn, that
    /// creature can't be regenerated this turn" (Runesword).
    GrantDamageDeniesRegenerationThisTurn { what: Selector },

    /// CR 603.4 — "When `what` leaves the battlefield this turn, [body]"
    /// (Runesword's sacrifice rider). Expires at cleanup.
    WhenTargetLeavesBattlefieldThisTurn { what: Selector, body: Box<Effect> },

    /// CR 702.79a / 702.93a — a persist / undying trigger resolving: the
    /// source card returns from its owner's graveyard under its owner's control
    /// with a `kind` counter. Pushed by the death funnels, never printed.
    PersistUndyingReturn { kind: crate::card::CounterType },

    /// CR 603.4 — "When `a` leaves the battlefield this turn, sacrifice `b`.
    /// When `b` leaves the battlefield this turn, sacrifice `a`" (Phantasmal
    /// Mount): two delayed triggers, each bound to the other permanent.
    SacrificeEachWhenOtherLeavesThisTurn { a: Selector, b: Selector },

    /// CR 614 — Frankenstein's Monster: as it enters, exile `count` graveyard
    /// creature cards or go to the graveyard instead; per card a chosen +2/+0,
    /// +1/+1 or +0/+2 counter.
    EnterExilingGraveyardCreaturesForCounters { count: Value },

    /// CR 614 — "As this enters, pay any amount of life", capped by `max`
    /// (Nameless Race). The paid amount is stamped on the source's
    /// `chosen_number` for `DynamicPt::ChosenNumberAsEntered` to read.
    PayAnyAmountOfLifeCapped { max: Value },

    /// "As this enters, choose a [filter] permanent", stored as the source's
    /// `chosen_permanent` (Dauntless Bodyguard); none legal, `None`.
    ChoosePermanentForSource { filter: SelectionRequirement },

    /// CR 201.3 — Academic Probation: name a nonland card; opponents can't cast
    /// it until your next turn (`opponents_cant_cast_named`).
    NameOpponentCastLock,

    /// Reflector Mage: the name of `what` (usually the bounced `Target(0)`) goes
    /// in the controller's `opponents_cant_cast_named` lock until their next turn
    /// — Academic Probation's lock, keyed off a target.
    LockTargetNameUntilYourNextTurn { what: Selector },
    /// "Your opponents can't cast spells with mana value `mv` until your next
    /// turn" (Living Breakthrough): recorded in the controller's
    /// `opponents_cant_cast_mv`, cleared as that player's next turn begins.
    LockOpponentsManaValueUntilYourNextTurn { mv: Value },

    /// "[Player] skips their next `count` turns": bumps `skip_turns`, consumed by
    /// `do_cleanup`'s turn advance (Ral Zarek, Guest Lecturer's -7 with
    /// `FlipCoin`).
    SkipTurns { who: PlayerRef, count: Value },
    /// CR 506 — `who` skips their next combat phase (`skip_next_combat`,
    /// consumed by `advance_step`). Stonehorn Dignitary.
    SkipNextCombatPhase { who: PlayerRef },
    /// CR 726 — `who` becomes the monarch. "You become the monarch."
    BecomeMonarch { who: PlayerRef },
    /// CR 726 — "`who` can't become the monarch this turn" (Jared Carthalion,
    /// True Heir): every way of becoming it is skipped until cleanup.
    CantBecomeMonarchThisTurn { who: PlayerRef },
    /// CR 701.54 — the Ring tempts `who`: temptation count +1 (max 4) and a
    /// Ring-bearer chosen. Bearer abilities are read off the level rather than
    /// an emblem.
    RingTempts { who: PlayerRef },
    /// CR 701.54c — each `what` creature is sacrificed at end of combat (via
    /// `attacking_token_cleanup`): the Ring-bearer's level 3.
    SacrificeAtEndOfCombat { what: Selector },
    /// CR 702.131 — Ascend: with ten or more permanents `who` gets the city's
    /// blessing. One-shot here; the static form re-checks.
    Ascend { who: PlayerRef },
    /// CR 732 — "it becomes day." Sets the game's day designation.
    BecomeDay,
    /// CR 732 — "it becomes night." Sets the game's night designation.
    BecomeNight,
    /// CR 500.7 — extra turn(s) after this one (`extra_turns`, consumed by
    /// `advance_turn`). Time Walk.
    TakeExtraTurn { who: PlayerRef, count: Value },
    /// "During that turn, power-up abilities can't be activated" (Kang the
    /// Conqueror): marks `who`'s newest banked extra turn (CR 500.7).
    PowerUpLockedDuringExtraTurn { who: PlayerRef },
    /// Proteus Staff: bottom the target creature; its controller reveals until a
    /// creature card, deploys it, bottoms the rest.
    BottomThenRevealUntilCreature { what: Selector },
    /// "Each player exiles the top card of their library. The player who
    /// exiled the card with the greatest mana value takes an extra turn after
    /// this one." Ties re-run over the tied players (Timesifter).
    ExileTopGreatestManaValueTakesExtraTurn,
    /// Quicksilver Elemental: gain `from`'s activated abilities this turn
    /// (`granted_activated_eot`), or give them to each `to` permanent (Grell
    /// Philosopher).
    GainAllActivatedAbilitiesOf {
        what: Selector,
        duration: Duration,
        #[serde(default)]
        to: Option<Selector>,
        /// "You may spend `color` mana as though it were mana of any color to
        /// activate those abilities" (Grell Philosopher's blue).
        #[serde(default)]
        spend_as_any: Option<crate::mana::Color>,
    },
    /// CR 701.12 — Confusion in the Ranks: exchange control with a permanent of
    /// another player sharing a type, picked at resolution (auto: priciest).
    ExchangeControlWithSharedType { what: Selector },
    /// "Search your library for a nonland card and reveal it. Each opponent who
    /// cast a spell this turn with the same name as that card loses `amount`
    /// life. Then shuffle." (Grim Reminder.)
    SearchRevealPunishSameNameCasters { amount: Value },
    /// CR 724.1 — "You control target player during that player's next turn."
    /// Registers a pending control entry consumed when that player actually
    /// takes a turn (Mindslaver).
    ControlPlayerNextTurn { who: PlayerRef },
    /// CR 724.2 — "You control target opponent during their next combat
    /// phase" (Secret of Bloodbending): control for that one combat phase only.
    ControlPlayerNextCombat { who: PlayerRef },
    /// CR 724.1 — "The first player controls the second player during the
    /// second player's next turn, and the second player controls the first
    /// player during the first player's next turn" (Cruel Entertainment).
    PlayersControlEachOthersNextTurn { first: PlayerRef, second: PlayerRef },
    /// CR 500.8 — "That player skips each instance of the chosen step or phase
    /// this turn." The affected player picks draw step / main phase / combat
    /// phase (`Decision::ChooseModes`); the pick is turn-scoped. Fatespinner.
    ChooseStepToSkipThisTurn { who: PlayerRef },
    /// CR 725 — end the turn: exile the stack, remove everything from combat,
    /// jump to cleanup. Sundial of the Infinite.
    EndTheTurn,
    /// CR 725.2 — end the combat phase: exile the stack, clear combat, go to the
    /// next main without end of combat (724.2e). No-op outside combat. Mandate
    /// of Peace.
    EndTheCombatPhase,
    /// CR 614 — this turn, a land tapped for mana makes something else instead.
    /// `mine_only`: your lands (Harvest Mage); `nonbasic_only` (Pale Moon).
    ReplaceLandManaThisTurn {
        #[serde(default)]
        mine_only: bool,
        #[serde(default)]
        nonbasic_only: bool,
        /// The tapping player picks a color instead of getting {C}.
        #[serde(default)]
        color_of_choice: bool,
        /// Only lands of this type (Chaos Moon's Mountains).
        #[serde(default)]
        land_type: Option<crate::card::LandType>,
    },
    /// CR 615.7 — prevent all damage a chosen `filter` source (battlefield or
    /// stack; auto: a stack spell, else the strongest permanent) would deal this
    /// turn (Burrenton Forge-Tender). `gain_life_from_colors`: Samite
    /// Ministration's refund (empty = none).
    PreventAllDamageFromChosenSourceThisTurn {
        filter: crate::card::SelectionRequirement,
        gain_life_from_colors: Vec<crate::mana::Color>,
    },
    /// CR 615.8 — prevent the next damage a chosen source would deal you this
    /// turn, then `rider` (New Way Forward) with the amount as
    /// `TriggerEventAmount` and the source as `Target(0)`.
    PreventNextDamageToYouFromChosenSourceWithRider {
        filter: crate::card::SelectionRequirement,
        rider: Box<Effect>,
    },
    /// Dismantle: put `amount` +1/+1 *or* charge counters on your `onto`
    /// permanent, kind and recipient picked at resolution (UI: `ChooseModes` +
    /// `ChooseTarget`; others: first kind, priciest recipient). No-op at zero or
    /// with no candidate.
    AddCountersOfChosenKind {
        onto: crate::card::SelectionRequirement,
        kinds: Vec<CounterType>,
        amount: Value,
    },
    /// CR 615.7 — prevent all damage `what` (permanent or stack spell) would
    /// deal this turn; targeted `PreventAllDamageFromChosenSourceThisTurn`.
    /// `gain_life`: gain the prevented amount, per event (Hallow).
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
    /// CR 615.7 — prevent all but 1 of the next damage a chosen [filter] source
    /// would deal you this turn (Forcefield): a one-event `all_but: 1` shield.
    PreventAllButOneFromChosenSource { filter: crate::card::SelectionRequirement },
    /// CR 615.7 — prevent the next damage a chosen [filter] source would deal you
    /// this turn: a one-event shield (Circles of Protection).
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
        /// "Exile cards from the top of your library equal to the damage
        /// prevented this way" (Bone Mask).
        #[serde(default)]
        exile_top_per_prevented: bool,
    },
    /// CR 615.7 — Refraction Trap: prevent the next `amount` from a chosen source
    /// to you and your permanents (one shared pool), dealing that much to `to`.
    PreventNextFromChosenSourceToTeam {
        amount: Value,
        to: Selector,
        /// "The next time [it] would deal damage" — soak one whole damage
        /// event rather than a point budget (Opal-Eye, Konda's Yojimbo).
        #[serde(default)]
        one_event: bool,
        /// "If damage from a [colour] source is prevented this way, you gain
        /// that much life" (Shadowbane). Empty = no life-gain rider; a
        /// non-empty list gates it on the damage source's colours.
        #[serde(default)]
        gain_life_colors: Vec<Color>,
    },
    /// CR 615.7 — prevent the next damage a chosen source would deal to any
    /// target this turn: the shield covers every recipient, unlike
    /// `PreventNextFromChosenSourceToTeam` (Martyr's Cause). `what` names the
    /// source instead of asking (Impulsive Maneuvers).
    PreventNextEventFromChosenSourceAnywhere {
        #[serde(default)]
        what: Option<Selector>,
        /// CR 614.9 — "that damage is dealt to that source's controller
        /// instead" (Reflect Damage). The shield redirects rather than
        /// simply preventing.
        #[serde(default)]
        reflect: bool,
    },
    /// "Creatures `what` gain protection from the colors of `of` until
    /// `duration`" (Samite Elder). Reads the live colors of the permanent(s)
    /// `of` resolves to and grants one `Keyword::Protection` per color.
    GrantProtectionFromColorsOf { what: Selector, of: Selector, duration: Duration },
    /// Skyship Weatherlight: search any number of `filter` cards, exile them
    /// linked to this, shuffle (`Selector::ExiledWithSource`,
    /// [`Effect::ReturnRandomExiledWithSource`]).
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
    /// CR 614.9 — the next damage to `what` this turn is dealt to `to` instead
    /// (Mirrorwood Treefolk). `chosen_source`: only from a chosen source (Jade
    /// Monolith); `whole_turn`: all damage this turn, not consumed (Kor Chant).
    RedirectNextDamageTo {
        what: Selector,
        to: Selector,
        #[serde(default)]
        chosen_source: bool,
        #[serde(default)]
        whole_turn: bool,
    },
    /// Goblin Game: everyone secretly hides 1+ items (`ask_seat_amount`; auto 1),
    /// loses that much life, and the fewest lose half their life rounded up.
    GoblinGame,
    /// Noxious Vapors: each player keeps one card per color in hand (a gold card
    /// covers all of its colors) and discards other nonlands.
    EachPlayerKeepsOneOfEachColorDiscardsRest,
    /// "Each player chooses a land they control of each basic land type.
    /// Return those lands to their owners' hands" (Planar Overlay).
    EachPlayerReturnsALandOfEachBasicType,
    /// Parley: each player reveals their top card; `then` per nonland revealed
    /// (`Value::CardsRevealedThisEffect`), then each draws (the revealed cards).
    Parley { then: Box<Effect> },
    /// "Reveal any number of `filter` cards in your hand, then [`then`]" — the
    /// Urza's Destiny Scent / Seer cycle and Metalworker. The revealed count is
    /// published as `Value::CardsRevealedThisEffect` for the body to read.
    RevealAnyNumberFromHand { filter: crate::card::SelectionRequirement, then: Box<Effect> },
    /// The Urza's Destiny name-hate cycle (Eradicate, Quash): exile `what` (Quash
    /// counters it) and every same-named card from its controller's graveyard,
    /// hand and library; they shuffle.
    ExileAllCopiesOfTargetName { what: Selector },
    /// Ancient Vendetta: name a card; exile up to `count` of it from `who`'s
    /// graveyard, hand and library; they shuffle.
    NameCardThenExileFromZones { who: PlayerRef, count: u32 },
    /// Dual Nature: exile all tokens named like `what` (read from LKI if gone);
    /// nontokens untouched. `same_controller`: only `what`'s controller's tokens
    /// (Legions to Ashes).
    ExileTokensSharingNameWith {
        what: Selector,
        #[serde(default)]
        same_controller: bool,
    },
    /// Exile `what` and return it under its owner's control (Ghostly Flicker).
    /// Tokens cease to exist (CR 111.7).
    ///
    /// ⚠ CR 400.7's "new object" holds for everything a card reads (tapped,
    /// counters, attachments, sickness, damage) but not the `CardId`: the same
    /// instance goes out and back, so id-keyed state (delayed triggers,
    /// `exiled_with`, held references) still matches. Harmless for friendly
    /// self-blinks; a card blinking to *break* such a link needs a fresh id.
    ExileAndReturnToOwner { what: Selector },
    /// CR 120.4a — `amount` damage to `to`, excess to `excess_to`, split before
    /// the event so the creature takes exactly lethal (marked damage and
    /// deathtouch counted). `condition` gates the rider (Ram Through's trample);
    /// `None` always redirects.
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
    /// Sword-Point Diplomacy: reveal the top `count`; each card goes to hand
    /// unless an opponent pays `life` (asked per card in turn order via
    /// `ask_seat_bool`); the rest are exiled. `payer` asks one player only
    /// (Protection Racket).
    RevealTopPayOrTake {
        count: Value,
        life: Value,
        #[serde(default)]
        payer: Option<PlayerRef>,
    },
    /// CR 715.4 — exile this Saga and return it transformed under your control,
    /// a new object (lore counters clear, back-face ETB fires) — Fable of the
    /// Mirror-Breaker III. With `from_graveyard`, returns it from a graveyard
    /// (Garland, Knight of Cornelia).
    ExileSelfReturnTransformed,
    /// LCI Ojer gods: "when this dies, return it tapped and transformed under
    /// its owner's control" — from the graveyard only, to the owner (CR 400.3),
    /// unlike `ExileSelfReturnTransformed`.
    ReturnSelfTransformedTappedToOwner,
    /// The mirror of `ExileSelfReturnTransformed` — "exile this, then return it
    /// to the battlefield (front face up)". Used by the FIN Dominants' Saga
    /// chapter III to reset the flip cycle.
    ExileSelfReturnFrontFace,
    /// CR 500.8 — additional combat phase(s) after this one: banks `count` on
    /// `GameState.additional_combat_phases`, looping End of Combat back to Begin
    /// Combat. For in-combat extra combats (Hellkite Charger), usually with an
    /// `Untap`. Main-phase "combat + main" sorceries use
    /// `AdditionalCombatPhaseAfterMain`.
    AdditionalCombatPhase { count: Value },
    /// Master Warcraft: this turn you declare attackers and blockers
    /// (`GameState.combat_chooser`); cleared at cleanup.
    ChooseCombatThisTurn,
    /// "You choose which creatures block this turn and how those creatures
    /// block" (Berserker's Frenzy) — `ChooseCombatThisTurn`'s block half: the
    /// resolving controller submits every block declaration this turn.
    ChooseBlocksThisTurn,
    /// Odric, Master Tactician: `ChooseBlocksThisTurn` ending with this combat.
    ChooseBlocksThisCombat,
    /// CR 500.8 — "after this main phase, an additional combat phase followed by
    /// an additional main phase": the banked combat begins as the current main
    /// phase ends; the main follows normally (Relentless Assault).
    AdditionalCombatPhaseAfterMain { count: Value },
    /// Last Night Together: at the start of the combat its
    /// `AdditionalCombatPhaseAfterMain` banked, your creatures other than the
    /// spell's targets (read at resolution) can't attack (CR 508.1c).
    OnlyTargetsCanAttackNextCombat,
    /// **Runtime-only** — the delayed half of `OnlyTargetsCanAttackNextCombat`:
    /// each creature you control not in `except` can't attack this combat.
    CantAttackThisCombatExcept { except: Vec<crate::card::CardId> },
    /// CR 500.7 — "there is an additional end step after this step." Banks
    /// `count` extra end steps; when the active player leaves the End step with
    /// one banked, the turn loops back to another End step (Y'shtola Rhul).
    AdditionalEndStep { count: Value },
    /// CR 500.9 — "you get an additional upkeep step after this one." Banks
    /// `count` extra upkeep steps; when the active player leaves the Upkeep
    /// with one banked, the turn loops back to another Upkeep (Paradox Haze).
    AdditionalUpkeepStep { count: Value },
    /// CR 500.8 — "there is an additional beginning phase after this phase"
    /// (Sphinx of the Second Sun), banked on the game.
    AdditionalBeginningPhase { count: Value },
    /// "At the beginning of each combat this turn, [body]": a turn-scoped
    /// `DelayedKind::EachCombatThisTurn`, expiring at cleanup (Full Throttle,
    /// with `AdditionalCombatPhaseAfterMain`).
    AtEachCombatThisTurn { body: Box<Effect> },
    /// CR 500.8 — "At the beginning of that combat, [body]" after an
    /// `AdditionalCombatPhaseAfterMain` in the same resolution: a one-shot
    /// delayed trigger bound to the combat just banked (Moraug).
    AtTheAddedCombat { body: Box<Effect> },
    /// CR 114 — `who` gets an emblem (named after its source) whose triggered
    /// abilities fire from the command zone. Planeswalker ultimates (Professor
    /// Dellian Fel).
    CreateEmblem {
        who: PlayerRef,
        name: String,
        #[serde(default)]
        triggered: Vec<TriggeredAbility>,
        /// Static (anthem) abilities the emblem grants — Vivien Reid's −8.
        #[serde(default)]
        statics: Vec<crate::card::StaticAbility>,
    },

    /// CR 603.7c — "until end of turn, whenever [event], [effect]" (Rite of
    /// Harmony, Bonus Round): the controller gets a this-turn emblem with
    /// `triggered`, dropped at cleanup.
    TriggersThisTurn { triggered: Vec<TriggeredAbility> },
    /// CR 611.2c — a rule-modifying "this turn" effect of a resolved spell
    /// ("creatures can't block this turn", Barrage of Boulders): a this-turn
    /// emblem carrying `statics`, so creatures arriving later are bound too.
    StaticsThisTurn { statics: Vec<crate::card::StaticAbility> },

    /// CR 611.2b — "until your next turn, whenever [event], [effect]" (Don't
    /// Move): `TriggersThisTurn`'s emblem, dropped as the controller's next
    /// turn begins instead of at cleanup.
    TriggersUntilYourNextTurn { triggered: Vec<TriggeredAbility> },

    /// "You can cast only `more` more spells this turn" (Irencrag Feat): caps
    /// the controller's spells this turn at the count so far plus `more`.
    CapSpellsThisTurn { more: u32 },

    /// "[Player] wins the game" (Approach of the Second Sun, Felidar Sovereign):
    /// eliminates every other player so the SBA pass (CR 104.2a) declares the
    /// winner.
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

    /// "Prevent all combat damage this turn" (fogs, Owlin Shieldmage): sets
    /// `GameState.prevent_combat_damage_this_turn`, which
    /// `resolve_combat_damage_with_filter` reads to zero assignments (CR 615.1
    /// emulation; see the field). Cleared in `do_cleanup`.
    PreventAllCombatDamageThisTurn,
    /// CR 615 — "Prevent all combat damage that would be dealt by [filter]
    /// creatures this turn" (Hunter's Ambush's nongreen fog). The filtered
    /// sibling of `PreventAllCombatDamageThisTurn`.
    PreventAllCombatDamageByMatchingThisTurn { filter: crate::card::SelectionRequirement },

    /// CR 615.1 — fog except combat damage by `except` creatures (Inspire Awe):
    /// the global flag plus a per-dealer exception.
    PreventCombatDamageExceptDealtBy { except: SelectionRequirement },

    /// CR 615 — prevent combat damage to `who` this turn
    /// (`GameState.combat_damage_prevented_to_players_this_turn`). Druid's
    /// Deliverance.
    PreventAllCombatDamageToPlayerThisTurn { who: PlayerRef },
    /// Take the Bait — "prevent all combat damage that would be dealt to you
    /// and planeswalkers you control this turn".
    PreventAllCombatDamageToPlayerAndWalkersThisTurn { who: PlayerRef },
    /// CR 615 — prevent all damage to you this turn (Selfless Squire), emitting
    /// `GameEvent::DamagePrevented` per prevention.
    PreventAllDamageToPlayerThisTurn { who: PlayerRef },
    /// CR 615 — "Until your next turn, prevent all damage that would be dealt
    /// to you" (Morningtide's Light): the player-scoped shield cleared as
    /// that player's next turn begins (`Player.all_damage_prevented_until_next_turn`).
    PreventAllDamageToPlayerUntilTheirNextTurn { who: PlayerRef },
    /// CR 508.1d — "During `who`'s next turn, creatures that player controls
    /// attack [this] if able" (Gideon Jura's +2). Sets each resolved player's
    /// `attack_lure` to the source; the declaration enforces it.
    LureCreaturesToSourceNextTurn { who: Selector },
    /// CR 508.1d — "During `who`'s next turn, each creature that player
    /// controls attacks if able" (Rowan Kenrith's +2): a requirement on the
    /// player, so a creature they get later is bound too.
    AttackIfAbleDuringTheirNextTurn { who: Selector },
    /// CR 508.1d — "up to one target creature an opponent controls attacks [this]
    /// next turn if able" (Gideon, Battle-Forged +2): one-creature
    /// `LureCreaturesToSourceNextTurn`, on `creature_attack_lures`.
    LureTargetToSourceNextTurn { what: Selector },

    /// CR 701.21 — the source's controller sacrifices it unless they pay {X},
    /// X its live mana value (auto-tapping). Soul Tithe.
    SacrificeSourceUnlessPayManaValue,

    /// "When this enters, sacrifice it unless you pay [cost]" (Gateway Plaza):
    /// fixed-cost sibling of `SacrificeSourceUnlessPayManaValue` (a land's mana
    /// value is 0). Auto-tapped.
    SacrificeSourceUnlessPay { cost: crate::mana::ManaCost },

    /// "Shuffle any number of cards from hand into your library, then draw that
    /// many" (Credit Voucher), via `Decision::ChooseCards` (AutoDecider: none).
    /// `bottom` bottoms them instead (Valakut Awakening).
    ShuffleAnyNumberFromHandThenDraw {
        who: PlayerRef,
        #[serde(default)]
        bottom: bool,
    },

    /// Game Preserve: everyone reveals their top card; if all are creatures they
    /// all enter under their owners.
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
    /// "Destroy this creature unless you pay {1} for each [thing]" — the
    /// destroy sibling (regeneration and indestructible apply): Musician's
    /// granted upkeep toll.
    DestroySourceUnlessPayValue { generic: Value },
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

    /// CR 614.9 — Maze of Ith: no combat damage to or by `target` this turn
    /// (`GameState.combat_damage_prevented_creatures`).
    PreventAllCombatDamageInvolving { target: Selector },

    /// CR 615 — prevent all combat damage dealt *to* `target` this turn (it still
    /// deals its own) — Fleeting Flight. Turn-scoped `PreventAllCombatDamageToThis`
    /// via `GameState.combat_damage_prevented_to_this_turn`.
    PreventCombatDamageToTargetThisTurn { target: Selector },

    /// Azorius Ploy: no combat damage dealt by `target` this turn
    /// (`combat_damage_prevented_by_this_turn`), mirror of
    /// `PreventCombatDamageToTargetThisTurn`.
    PreventCombatDamageByTargetThisTurn { target: Selector },
    /// CR 615 — "Prevent all damage `target` would deal this turn", combat and
    /// noncombat alike (Chain of Silence). The all-damage superset of
    /// `PreventCombatDamageByTargetThisTurn`.
    PreventAllDamageByTargetThisTurn { target: Selector },

    /// CR 615 — "prevent all damage that would be dealt this turn by [filter]
    /// creatures" (Obscuring Haze).
    ///
    /// All-damage superset of
    /// [`PreventAllCombatDamageByMatchingThisTurn`](Self::PreventAllCombatDamageByMatchingThisTurn)
    /// and filtered sibling of
    /// [`PreventAllDamageByTargetThisTurn`](Self::PreventAllDamageByTargetThisTurn).
    /// Kept as a filter, not an id snapshot: a source is judged when the damage
    /// would be dealt, so control changes and later arrivals count.
    /// Controller-relative atoms read from the resolving seat.
    PreventAllDamageByMatchingThisTurn { filter: crate::card::SelectionRequirement },
    /// Mark that the source did something this turn, for
    /// `Predicate::SourceMarkedThisTurn` until cleanup (Thirsting Axe).
    MarkSourceThisTurn,
    /// CR 615.1 — prevent combat damage this turn to `filter` (Pack Leader),
    /// judged when dealt from the resolving seat, so later arrivals count and
    /// controller changes don't.
    PreventCombatDamageToMatchingThisTurn { filter: crate::card::SelectionRequirement },

    /// "You may have `dealer` deal damage equal to its power to `to`. If you
    /// do, `dealer` assigns no combat damage this turn." The Laccolith cycle's
    /// becomes-blocked trigger; declining leaves combat untouched.
    MayDealPowerThenNoCombatDamage { dealer: Selector, to: Selector },

    /// Kozilek's Pathfinder: `target` can't block `source` this turn
    /// (`GameState.cant_block_pairs`).
    CantBlockSourceThisTurn { target: Selector },
    /// CR 509.1b — "creatures matching `filter` can't block this turn"
    /// (Concussive Bolt's metalcraft rider). Snapshots the matching creatures
    /// at resolution; cleared at cleanup.
    MatchingCantBlockThisTurn { filter: SelectionRequirement },

    /// CR 508.1a — may attack this turn as though without defender
    /// (`attack_despite_defender_this_turn`). Krotiq Nestguard.
    AttackDespiteDefenderThisTurn { what: Selector },

    /// CR 509.1c — "Target creature blocks `source` this turn if able."
    /// Sets the target's `must_block` to the ability source (like Provoke
    /// but without untapping the target). Matsu-Tribe Decoy.
    MustBlockSource {
        what: Selector,
        /// "Defending player chooses …" (Crashing Boars) — the named seat
        /// picks one of `what`'s matches instead of the engine taking the
        /// first. `None` forces every match, the Matsu-Tribe Decoy shape.
        #[serde(default)]
        chooser: Option<PlayerRef>,
    },

    /// CR 509.1c — the two-slot sibling: "`blocker` blocks `attacker` this
    /// turn if able", where the attacker is itself chosen (Feral Contest).
    MustBlockTarget { blocker: Selector, attacker: Selector },

    /// CR 508.1d — "`attacker` attacks `defender` this turn if able" (Dulcet
    /// Sirens): stamps the player on the creature's `chosen_player` and grants
    /// `Keyword::MustAttackChosenPlayer` until end of turn.
    MustAttackPlayerThisTurn { attacker: Selector, defender: Selector },
    /// `MustAttackPlayerThisTurn` for `duration` — Silver Surfer's "until the
    /// end of your next turn, target creature attacks that player each combat
    /// if able".
    MustAttackPlayerFor { attacker: Selector, defender: Selector, duration: Duration },
    /// "Other creatures you control attacking that player get +P/+T until end
    /// of turn", that player being the one the source attacks (Namor,
    /// Atlantean King).
    PumpOtherAttackersOnSamePlayer { power: Value, toughness: Value },
    /// Exile a card matching `filter` chosen at random from `who`'s graveyard,
    /// linked to the source (`Selector::CardExiledWithSource`) — Power Pack.
    ExileRandomFromGraveyardWithSource { who: PlayerRef, filter: SelectionRequirement },

    /// "Destroy `what`. For each permanent put into a graveyard this way, its
    /// controller creates a token" (Terastodon). Victims that survive the
    /// destroy (indestructible, a replacement) pay nothing.
    DestroyThenVictimControllersMakeToken {
        what: Selector,
        definition: std::sync::Arc<crate::card::TokenDefinition>,
        /// "They can't be regenerated" (March of Souls). Defaults to false
        /// (Terastodon lets its victims regenerate).
        #[serde(default)]
        no_regen: bool,
    },

    /// CR 615.7 — prevent the next N damage to `target` this turn (Samite
    /// Healer): a per-target shield expiring at cleanup.
    PreventNextDamage { target: Selector, amount: Value },
    /// Test of Faith: prevent the next N damage to `target` this turn, a +1/+1
    /// counter per point prevented (`counters_on_target`). `counter` overrides
    /// the kind (Sacred Boon's +0/+1).
    PreventNextDamageWithCounters {
        target: Selector,
        amount: Value,
        #[serde(default)]
        counter: Option<crate::card::CounterType>,
    },
    /// CR 615.7 — Barbed Wire: prevent the next N damage this permanent deals
    /// this turn, a shield keyed to the source.
    PreventNextDamageFromSourceThisTurn { amount: Value },
    /// CR 615 — "The next time a source of your choice would deal damage to
    /// you this turn, prevent half that damage, rounded down" (Dark Sphere).
    PreventNextHalfDamageToYouThisTurn,
    /// CR 615 — Eye for an Eye: the next damage a chosen source deals you this
    /// turn also deals that much to its controller.
    MirrorNextDamageToYouThisTurn,
    /// CR 614.9 — "Until end of turn, if damage would be dealt to any
    /// creature, you may have that damage dealt to you instead" (Blood of the
    /// Martyr). Set on the effect's controller; cleared at end of turn.
    RedirectCreatureDamageToYouThisTurn,

    /// CR 614.9 — Carom: the next `amount` to `target` this turn is dealt to `to`
    /// instead (a shield flagged `redirect_to`).
    RedirectNextDamage { target: Selector, to: Selector, amount: Value },

    /// CR 614.9 — "The next time `what` would deal combat damage to `to` this
    /// turn, `what` deals that damage to itself instead" (Shield Dancer). A
    /// one-event shield on `to`, scoped to `what` and redirecting back onto it.
    RedirectNextDamageBackAtSource { what: Selector, to: Selector },

    /// CR 615 — prevent all damage to `target` this turn (Pradesh Gypsies), a
    /// one-target fog. `redirect_to` deals it there instead (CR 614.9 — Sivvi's
    /// Valor).
    PreventAllDamageThisTurn {
        target: Selector,
        #[serde(default)]
        redirect_to: Option<Selector>,
    },
    /// "Prevent all damage that would be dealt to `target` this turn. For each
    /// 1 damage prevented this way, put a +1/+1 counter on it." Brace for
    /// Impact — the counter rider rides the shield (`counters_on_target`).
    PreventAllDamageThisTurnWithCounters { target: Selector },
    /// CR 615 — Comeuppance: prevent damage to `target` this turn from sources
    /// you don't control; this deals that much to a creature source, or to a
    /// noncreature source's controller, as a new event.
    PreventOpposingDamageThisTurnReflecting { target: Selector },

    /// CR 615 — "Prevent all damage that `from` would deal to `to` this turn."
    /// A source-restricted fog (Stonewise Fortifier's "prevent all damage that
    /// would be dealt to this creature by target creature this turn").
    PreventAllDamageBetweenThisTurn { from: Selector, to: Selector },
    /// CR 614.9 — "all damage that would be dealt to `to` this turn by `from`
    /// is dealt to `redirect_to` instead" (Shimian Night Stalker; Mirror
    /// Strike's "to its controller").
    RedirectAllDamageBetweenThisTurn { from: Selector, to: Selector, redirect_to: Selector },

    /// "You may tap or untap `what`" — the controller picks which per resolved
    /// permanent (Thassa's Ire, Puppeteer). Tapped permanents untap and
    /// untapped ones tap when the controller declines to choose.
    TapOrUntap { what: Selector },

    /// CR 615 — Kiora, the Crashing Wave +1: until your next turn, no damage to
    /// or by `target` (`damage_locked_until_turn_of`).
    PreventDamageToAndByUntilYourNextTurn { target: Selector },

    /// CR 701.6 — counter target spell only if a card exiled with the source
    /// shares its name (Mindreaver). A no-op when nothing matches.
    CounterSpellIfNameExiledWithSource { what: Selector },

    /// Kill-Suit Cultist: the next damage to `target` this turn destroys it
    /// instead (a one-event shield flagged `destroy`).
    ReplaceNextDamageWithDestroy { target: Selector },

    /// CR 615.1 — prevent the next damage to `target` this turn, and it gains
    /// that much life (Reverse Damage): a per-target shield flagged `gain_life`.
    PreventNextDamageAndGainLife { target: Selector, amount: Value },

    /// "Damage can't be prevented this turn." (CR 615.12) Sets a global
    /// flag that suppresses every prevention shield for the rest of the
    /// turn. Skullcrack, Heated Debate, Impractical Joke's rider.
    DamageCantBePreventedThisTurn,
    /// CR 615.12 — "The damage can't be prevented" scoped to the damage
    /// `body` deals (Volcano Hellion's entry), not the rest of the turn.
    DamageCantBePrevented { body: Box<Effect> },

    /// "Players can't search libraries this turn." (Shadow of Doubt.) Sets a
    /// global flag; library searches that turn find nothing.
    PreventSearchesThisTurn,

    /// "`who` gains protection from everything until their next turn"
    /// (The One Ring): can't be targeted, all damage prevented. Cleared
    /// when that player's turn begins.
    PlayerProtectionUntilNextTurn { who: PlayerRef },

    /// "When [the token created this resolution] leaves, [body]": a delayed
    /// trigger with the current trigger source captured as `Target(0)` (Hofri
    /// Ghostforge).
    WhenLastCreatedTokenLeaves { body: Box<Effect> },

    /// Crippling Fear: choose a creature type (`ChooseCreatureType`, caster's
    /// seat); every other creature gets `PumpPT(power, toughness, EOT)`. Asked
    /// synchronously off `self.decider` (AutoDecider: `Demon`); UI players get no
    /// prompt yet.
    DiminishCreaturesExceptChosenType { power: Value, toughness: Value },

    /// CR 615 — "Prevent all damage that would be dealt to `target` this turn
    /// by sources of the color of your choice." Avacyn, Guardian Angel. The
    /// color is picked as the effect resolves (`Decision::ChooseColor`).
    PreventAllDamageFromChosenColorThisTurn { target: Selector },

    /// Boonweaver Giant: search graveyard, hand and/or library for an Aura and
    /// attach it to the source (shuffle if the library was searched). The
    /// enchant filter isn't re-checked.
    SearchAuraAttachToSource,
    /// Light-Paws, Emperor's Voice: search for an Aura with mana value ≤ `cap`
    /// and a name unlike your Auras that can enchant the source; attach it;
    /// shuffle.
    SearchAuraAttachToSourceCappedBy { cap: Selector },

    /// CR 303.4f / 301.5c — put each `filter` card from the controller's `zones`
    /// (in order, priciest first, at most `max`) onto the battlefield attached to
    /// `host`, or with `host: None` to its own best legal host (a creature with
    /// `creatures_only`). No legal host: it stays (CR 303.4i). Retether.
    PutOntoBattlefieldAttached {
        zones: Vec<crate::card::Zone>,
        filter: SelectionRequirement,
        host: Option<Selector>,
        max: Option<Value>,
        #[serde(default)]
        creatures_only: bool,
        /// An Equipment enters unattached (Brotherhood Outcast's return);
        /// follow with `attach_moved_equipment_to_your_creature` for "you may
        /// attach it to a creature you control" (Vault 101: Birthday Party).
        #[serde(default)]
        equipment_unattached: bool,
    },
    /// Reveal the top `count` cards; each `filter` card among them goes onto
    /// the battlefield attached to its best legal host; the rest go to the
    /// bottom in a random order (Knickknack Ouphe).
    RevealTopPutAttached { count: Value, filter: SelectionRequirement },
    /// Reveal until a `filter` card; put it onto the battlefield attached to
    /// its best legal host, else into your hand; the rest go to the bottom in
    /// a random order (Songbirds' Blessing).
    RevealUntilPutAttachedElseHand { filter: SelectionRequirement },
    /// "The next spell you cast this turn has convoke" (CR 702.51 — Wand of
    /// the Worldsoul, Flockchaser Phantom).
    NextSpellGainsConvokeThisTurn,
    /// "The next spell you cast this turn has improvise" (CR 702.126 —
    /// Archway of Innovation). The artifact sibling of
    /// `NextSpellGainsConvokeThisTurn`.
    NextSpellGainsImproviseThisTurn,
    /// "The next instant or sorcery spell you cast this turn has storm"
    /// (CR 702.40 — Storm, Force of Nature's Ceaseless Tempest).
    NextInstantOrSorceryGainsStormThisTurn,
    /// CR 500.4 exception — "until end of turn, you don't lose unspent
    /// `color` mana as steps and phases end" (The Last Agni Kai). Ends in the
    /// cleanup step with the turn's other until-end-of-turn effects.
    KeepUnspentColorManaThisTurn { who: PlayerRef, color: crate::mana::Color },
    /// CR 613.1c / 612 — Deadpool, Trading Card: `a` and `b` swap text boxes
    /// (keywords and abilities) while on the battlefield (CR 400.7). Name, types,
    /// cost and P/T stay. No-op unless both are permanents.
    ExchangeTextBoxes { a: Selector, b: Selector },
    /// For each of `keywords` a creature card in your graveyard has, a counter
    /// of it on a creature you control; then a +1/+1 counter on the source per
    /// counter placed (Kathril, Aspect Warper).
    KeywordCountersFromGraveyard { keywords: Vec<crate::card::Keyword> },
    /// `what` gains until end of turn each of `keywords` that a creature you
    /// control has (Majestic Myriarch).
    GainKeywordsYourCreaturesHave { what: Selector, keywords: Vec<crate::card::Keyword> },
    /// Reveal the top `count`; choose a different card for each of `keywords`
    /// it has; one chosen permanent card onto the battlefield, the other
    /// chosen cards to hand, the rest to the graveyard (Selective Adaptation).
    RevealTopChooseByKeyword { count: Value, keywords: Vec<crate::card::Keyword> },
    /// Move one counter of any kind from `from` onto `to` (Nesting Grounds).
    MoveOneCounter { from: Selector, to: Selector },
    /// "As this enters, choose one of `options`" — stamps
    /// `CardInstance.chosen_card_type` (Archon of Valor's Reach).
    ChooseCardTypeAmongForSource(Vec<crate::card::CardType>),
    /// Reveal your top card; a nonland card with mana value less than the
    /// triggering spell's may be cast free, else it goes to hand (Rashmi,
    /// Eternities Crafter).
    RevealTopCastFreeIfLesserElseHand,
    /// `who` reveals until a creature card; the source becomes a copy of it
    /// until end of turn; the revealed cards go to the bottom in a random
    /// order (Nascent Metamorph).
    RevealUntilCreatureBecomeCopy { who: PlayerRef },
    /// Reveal your top `count` cards; put every land card among them onto
    /// the battlefield (`tapped` when set), the rest on the bottom in a random
    /// order (Animist's Awakening).
    RevealTopPutLandsRestBottomRandom { count: Value, tapped: bool },
    /// Look at your top card; if it's a land card, or a creature card with
    /// mana value at most `max_mv`, you may put it onto the battlefield
    /// (Nissa, Steward of Elements' 0).
    LookTopMayPutLandOrCreatureMvAtMost { max_mv: Value },
    /// Exile up to one card of each card type from `who`'s graveyard; put a
    /// +1/+1 counter on the source for each card exiled (Grime Gorger).
    ExileOnePerCardTypeFromGraveyardGrow { who: PlayerRef },
    /// The damaging creature's controller gains control of one of the damaged
    /// player's contested lands and untaps it (Turf War).
    TakeContestedLand,
    /// Exile each resolved spell from the stack (not a counter), linked to
    /// the source (Smirking Spelljacker).
    ExileSpellLinked { what: Selector },
    /// Look at the top `count` of `who`'s library; exile `take` face down,
    /// playable with any mana type while exiled; the rest to the graveyard
    /// (`rest_to_graveyard`, Thief of Sanity) or bottom. The engine picks.
    /// `any_color`: any *color* only, so {C} pips still want colorless (Siphon
    /// Insight).
    LookTopExileOneFaceDownMayPlay {
        who: PlayerRef,
        count: Value,
        rest_to_graveyard: bool,
        #[serde(default)]
        any_color: bool,
        /// How many of them are exiled (Black Cat's "exile two of them").
        #[serde(default = "one_u32")]
        take: u32,
    },
    /// `who` chooses X cards from their hand; you may cast one of them free
    /// (Extract Brain). Both picks are the engine's.
    OpponentChoosesXFromHandCastOneFree { who: PlayerRef },
    /// `who` exiles their top card; if it's nonland, you may cast it free
    /// (Mind's Dilation).
    ExileTopMayCastFreeIfNonland { who: PlayerRef },
    /// Each player exiles their top card; until end of turn you may play one
    /// of them (`OneCastAmongGranted`), paying life rather than mana for a
    /// spell (Nashi, Moon Sage's Scion).
    ExileTopOfEachLibraryMayPlayForLife,
    /// CR 701.40 — manifest the top card of `who`'s library under your
    /// control (Thieving Amalgam, Orochi Soul-Reaver).
    ManifestTopOfLibraryUnderYou { who: PlayerRef },
    /// Counter every triggered or activated ability on the stack controlled by
    /// a player `who` resolves to (Kadena's Silencer).
    CounterAllAbilitiesOf { who: PlayerRef },
    /// "Counter all spells your opponents control and all abilities your
    /// opponents control. Create a [token] for each spell and ability
    /// countered this way." (Glen Elendra's Answer.)
    CounterOpponentsSpellsAndAbilities { token: std::sync::Arc<TokenDefinition> },
    /// CR 903.9 — `who` returns each commander they control from the
    /// battlefield to the command zone (Leadership Vacuum).
    ReturnCommandersToCommandZone { who: PlayerRef },
    /// Exchange control of the spell `a` and the creature `b`; the spell's
    /// new controller may choose new targets (Sudden Substitution).
    ExchangeSpellAndCreatureControl { a: Selector, b: Selector },
    /// "The next spell you cast this turn can be cast as though it had flash"
    /// (Ride the Avalanche). Spent by that cast.
    NextSpellHasFlashThisTurn,
    /// "The next creature card you play this turn can be played as though it
    /// had flash" (Scout's Warning). Spent by that creature's cast; a
    /// noncreature spell in between leaves it armed.
    NextCreatureSpellHasFlashThisTurn,
    /// "The next [filter] spell you cast this turn can be cast as though it
    /// had flash" (Quicken — the next sorcery). Spent by the first match.
    NextMatchingSpellHasFlashThisTurn { filter: crate::card::SelectionRequirement },
    /// Progenitor's Icon: your next spell of the chosen type this turn has
    /// flash.
    NextSpellOfChosenTypeHasFlashThisTurn,

    /// The Chain Veil: one more loyalty activation per planeswalker this turn
    /// (`extra_loyalty_activations`).
    GrantExtraLoyaltyActivations,

    /// Master of Predicaments: `who` guesses whether your chosen hand card's mana
    /// value exceeds `threshold` (`OptionalTrigger`, true = greater); wrong, you
    /// may cast it free.
    GuessManaValueAboveElseCastFree { who: PlayerRef, threshold: u32 },
    /// The Seventh Doctor: `GuessManaValueAboveElseCastFree` against a `Value`,
    /// with `otherwise` if nothing was cast.
    GuessManaValueAgainstValue { who: PlayerRef, threshold: Value, otherwise: Box<Effect> },

    /// "`who` may pay `life`. If they don't, they return a permanent they
    /// control to its owner's hand." Umbilicus — one pay-or-bounce decision per
    /// affected player, asked of that player.
    PlayerReturnsPermanentUnlessPaysLife { who: PlayerRef, life: u32 },

    /// Noetic Scales: bounce `who`'s creatures with power greater than their hand
    /// size, compared per player.
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

    /// CR 615 — Serra's Hymn: prevent the next `total` this turn divided among
    /// targets (`Decision::DivideDamage`), like `DealDamageDivided`.
    PreventNextDamageDivided { total: Value, filter: SelectionRequirement, max_targets: u8 },

    /// Stamp `what`'s first permanent on the source's `chosen_permanent` for
    /// `Selector::ChosenPermanentOfSource` — resolution-time
    /// `Effect::ChoosePermanentForSource` (Diabolic Servitude).
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

    /// Thieves' Auction: exile all nontoken permanents; from you in turn order,
    /// players claim one each onto the battlefield tapped until none are left.
    ThievesAuction,

    /// Stamp the player on the source's `chosen_player` for
    /// `ChosenPlayerOfSource` — player [`Effect::RememberPermanentOnSource`]
    /// (Torment Nightmare Horrors).
    RememberPlayerOnSource { who: PlayerRef },
    /// CR 508.1d — Ruhan of the Fomori: the source must attack `who` this combat,
    /// kept apart from `chosen_player` so it can't be overwritten.
    MustAttackPlayerThisCombat { who: PlayerRef },
    /// CR 614.12 — "as this enters, choose a player" (`opponent`: an opponent),
    /// stamped on `chosen_player` via a seat ballot led by the most hostile
    /// opponent. True-Name Nemesis.
    ChoosePlayerForSource { opponent: bool },
    /// Ertai's Meddling (CR 701.6a-adjacent): the spell's controller exiles it
    /// with `count` delay counters, one removed each upkeep; the last returns it
    /// to the stack (`process_delayed_spells`).
    ///
    /// Approximation: the return is a free cast from exile with an auto-picked
    /// target, not the original spell.
    ExileSpellWithDelayCounters { what: Selector, count: Value },
    /// CR 508.1a/d — Oracle en-Vec: next turn the player's chosen creatures must
    /// attack and others can't; chosen ones that didn't attack are destroyed at
    /// end step (`GameState.attack_mandates`).
    AttackMandateNextTurn { who: PlayerRef },

    /// CR 614 — Dracoplasm: as it enters, sacrifice any number of creatures; its
    /// P/T become their totals, stamped on `chosen_number` /
    /// `remembered_amount` for `DynamicPt::EnteredTotals`.
    AsEntersSacrificeForTotalPt,

    /// Carrion Rats: in turn order from you, the first player willing to exile
    /// `count` graveyard cards does; then `then` runs once.
    AnyPlayerMayExileFromGraveyard { count: Value, then: Box<Effect> },

    /// CR 614 — "Until end of turn, if `from` would draw a card, instead that
    /// player skips that draw and you draw a card" (Plagiarize). Registered on
    /// `GameState.draws_redirected_this_turn` and consumed in `draw_one`.
    RedirectDrawsThisTurn { from: PlayerRef },

    /// CR 615 — Equal Treatment: this turn, any damage of `at_least`+ becomes
    /// `becomes` (in `scale_damage_to`).
    DamageBecomesThisTurn { at_least: u32, becomes: u32 },

    /// Flaming Gambit: `amount` to a player or planeswalker; the victim may
    /// redirect it to their creature.
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

    /// "When you cast this spell, any player may pay `life` life; if one does,
    /// counter it" (Dash Hopes), on a `SpellCast / SelfSource` trigger. Only
    /// seats that can pay are asked (CR 119.4); headless policy in
    /// `game/effects/punisher_counter.rs`.
    AnyPlayerMayPayLifeToCounterThis { life: u32 },
    /// Shivan Wumpus: in turn order from you, the first player to sacrifice a
    /// [filter] puts this on top of its owner's library.
    AnyPlayerMaySacrificeToTopdeckSource { filter: SelectionRequirement },
    /// "Each player discards any number of cards, then draws that many
    /// cards" (Flux): APNAP choices, discards together, then each seat draws
    /// its OWN count.
    EachPlayerDiscardsAnyThenDrawsThatMany,
    /// "Each player shuffles all cards from their hand and all permanents they
    /// own into their library, then draws that many cards" (The Great Aurora).
    EachPlayerShufflesHandAndPermanentsThenDraws,
    /// "Each player chooses up to `keep` cards in their hand, then shuffles
    /// the rest into their library" (Worldpurge): APNAP choices, then each
    /// seat shuffles its own rest in.
    EachPlayerKeepsUpToFromHandShufflesRest { keep: u32 },
    /// "Starting with you, each player chooses up to `keep` permanents they
    /// control. All permanents other than this one that weren't chosen phase
    /// out" (Disciple of Caelus Nin) — asked in turn order (CR 101.4).
    EachPlayerKeepsUpToPermanentsRestPhaseOut { keep: u32 },
    /// "Roll a d`sides` twice. Its base power becomes the first result and its
    /// base toughness becomes the second" — an as-enters effect on the source
    /// (Vedalken Squirrel-Whacker): one indefinite layer-7b set it owns.
    RollDiceSetBasePt { sides: u8 },
    /// "[Who] may choose that …" (Soul Echo's upkeep): the seat `who` names
    /// is asked; yes stamps the source with this turn (`chosen_number`), no
    /// clears it. Read by `StaticEffect::ReplaceDamageToYouRemovingStampedCounters`.
    PlayerMayStampSourceTurn { who: Selector },
    /// Put one card exiled with the source that shares a name with the spell
    /// `what` resolves to into its owner's graveyard (Jester's Scepter's
    /// "put a card exiled with this artifact into its owner's graveyard").
    BinExiledWithSourceNamedLike { what: Selector },
    /// "[Player] loses all unspent mana" (Mana Short, Worldpurge) — an
    /// effect, so Upwelling's step-end exception doesn't stop it (CR 500.4);
    /// Yurlok's mana burn sees the loss.
    LoseUnspentMana { who: PlayerRef },
    /// Each permanent `what` names has exactly `subtypes` as its artifact
    /// subtypes from now on — "it isn't an Equipment" (Dancing Sword), an
    /// indefinite layer-4 rewrite.
    SetArtifactSubtypes { what: Selector, subtypes: Vec<crate::card::ArtifactSubtype> },
    /// "It becomes a Vehicle artifact" (Captain Rex Nebula): for `duration`,
    /// exactly `subtypes` as artifact subtypes and no creature or land types
    /// (CR 205.1a, with a `SetCardTypesFor` beside it); crewing adds Creature
    /// but no creature types.
    BecomeArtifactSubtypesFor { what: Selector, subtypes: Vec<crate::card::ArtifactSubtype>, duration: Duration },
    /// "Spells [who] casts this turn cost {`amount`} more to cast" (Academy
    /// Loremaster) — `Player.spell_tax_this_turn`, cleared at the turn
    /// boundary.
    SpellsCostMoreThisTurn { who: PlayerRef, amount: u32 },

    /// "Any opponent may" punisher: each `who` seat is asked `prompt` in turn
    /// order from the controller; the first yes runs `accepted` (that seat is
    /// [`PlayerRef::AcceptingPlayer`]) and stops; `otherwise` runs if all
    /// decline. Book Burning.
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

    /// CR 407.4 — `who` antes their top card. `optional` asks first; `then` runs
    /// per player who anted, as controller (Rebirth), `else_` per decliner
    /// (Amulet of Quoz).
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

    /// CR 614.12a — a modal choice made as a permanent enters, asked at
    /// resolution.
    ///
    /// [`Effect::ChooseMode`] can't: it reads `ctx.mode` from the stack item, and
    /// an as-enters replacement has none, so it silently takes mode 0. A `May*`
    /// node can't either: `AutoDecider` declines every `OptionalTrigger`, taking
    /// the downside for headless seats. Here `AutoDecider` answers mode 0, the
    /// bot scores modes by outcome, and a `wants_ui` seat suspends for a prompt.
    /// Shocklands, Lavabrink Venturer.
    AsEntersChooseMode(Vec<Effect>),

    /// CR 614.12 — the "if you don't, it enters tapped" branch of an as-enters
    /// replacement (shocklands). Unlike [`Effect::Tap`] it sets the tapped
    /// flag directly: entering tapped isn't *becoming* tapped, so no
    /// `PermanentTapped` event. Honours "lands enter untapped" (Spelunking) as
    /// `apply_enters_tapped_replacement` does. Only inside
    /// `CardDefinition::as_enters_effect`.
    SourceEntersTapped,

    /// Grave Consequences: each player may exile any graveyard cards (their own
    /// `Decision::ChooseCards`), then `then` runs once regardless — unlike
    /// [`Effect::AnyPlayerMayExileFromGraveyard`].
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

    /// "Note [amount]" (Sigarda's Splendor's life total): stamp the value on
    /// the source permanent (`CardInstance.remembered_amount`), replacing any
    /// earlier note; read back by [`Value::RememberedAmountOfSource`].
    NoteAmountOnSource { amount: Value },

    /// A damage trigger's "noncombat" gate (Smaug, the Impenetrable): rewritten
    /// per event at collection (`for_trigger_event`) into `body` for noncombat
    /// damage; the battlefield walk skips combat damage outright (CR 603.2).
    /// `EventKind` has no bit left for a noncombat kind.
    IfTriggerDamageNoncombat { body: Box<Effect> },

    /// Spelljack: counter the spell into exile; the counterer may play it free
    /// while it stays exiled.
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
    /// "`who` can't attack `defender` or planeswalkers `defender` controls
    /// this turn" (Sandswirl Wanderglyph) — a player-level ban (CR 508.1)
    /// covering every creature `who` controls; cleared at cleanup.
    CantAttackPlayerOrWalkersThisTurn { who: PlayerRef, defender: PlayerRef },
    /// "`who` can't attack you or permanents you control during their next
    /// turn" (The Second Doctor) — a player-level ban (CR 508.1), so a
    /// creature they get later is barred too.
    CantAttackYouDuringTheirNextTurn { who: PlayerRef },
    /// "Until that player's next turn, … their life total can't change"
    /// (Teferi's Reproach, CR 119.10).
    LifeLockUntilNextTurn { who: Selector },
    /// Dack Fayden, Helping Hand: pair the `what` permanents with your living
    /// opponents in turn order, each gaining control of theirs; extras stay put.
    DistributeControlAmongOpponents { what: Selector },
    /// The Ur-Sphinx: everyone mills `count` (from you), then you may cast one
    /// milled card free per player.
    EachPlayerMillsYouMayCastOne { count: Value },
    /// "Creatures `who` controls can't attack [subtype] planeswalkers
    /// `defender` controls this turn" (Jace, Multiverse Architect). Cleared
    /// at cleanup.
    CantAttackPlaneswalkerTypeThisTurn {
        who: PlayerRef,
        defender: PlayerRef,
        subtype: crate::card::PlaneswalkerSubtype,
    },

    /// "Prevent all damage [colors] sources would deal this turn" (Luminesce), or
    /// with `to_your_creatures` only to your creatures (Surge of Salvation). One
    /// turn-long shield per colour (CR 615.1); multicolour sources hit each.
    PreventDamageFromColorsThisTurn { colors: Vec<crate::mana::Color>, #[serde(default)] to_your_creatures: bool },
    /// Prismatic Strands: prevent all damage sources of a chosen color would deal
    /// this turn — the recipient-less
    /// [`Effect::PreventAllDamageFromChosenColorThisTurn`].
    PreventAllDamageFromChosenColorGlobally,

    /// Shaman's Trance: this turn, only you play from graveyards — anyone's, as
    /// your own.
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
    /// "At the beginning of your next declare attackers step" — fired once the
    /// declaration is made, attackers or none (CR 508.2; Noctis, Heir
    /// Apparent's warp-strike return).
    YourNextDeclareAttackersStep,
    /// "Until your next turn, whenever a creature attacks you or a
    /// planeswalker you control, …" (Jace, Architect of Thought's +1). The
    /// attacker is the trigger source.
    CreatureAttacksYouUntilYourNextTurn,
    YourNextUpkeep,
    /// "At the beginning of the next upkeep" — whoever's turn it is, and not
    /// the current one when registered during an upkeep (The Eagles Are
    /// Coming!).
    NextUpkeep,
    NextEndStep,
    /// "At the beginning of your next end step" (Desert Warfare): the
    /// controller's own end step — this turn's if it's theirs and it hasn't
    /// begun, else their next turn's. `NextEndStep` fires on anyone's.
    YourNextEndStep,
    /// "At the beginning of that turn's end step" for the extra turn this
    /// effect grants (Final Fortune): the controller's end step on a LATER
    /// turn, never this one's.
    YourNextTurnsEndStep,
    /// Until end of turn, whenever a permanent an opponent controls deals
    /// damage to you — that permanent is the trigger source (Hellish
    /// Rebuke). Fires every time, not once.
    OpponentPermanentDamagesYouThisTurn,
    /// "At the beginning of the end step of target player's next turn"
    /// (Suppress). `Effect::DelayUntil` reads the player from target slot 0.
    TargetsNextEndStep,
    /// "At the beginning of [the damaged player]'s next draw step" (Nafs Asp).
    /// Reads the player from the trigger's event subject, falling back to
    /// target slot 0.
    TargetsNextDrawStep,
    /// "At the beginning of that player's next upkeep" (Plague Reaver's
    /// return). Reads the player from target slot 0.
    TargetsNextUpkeep,
    /// "At end of combat, …" — fires once at the current turn's end-of-combat
    /// step (Fortune, Loyal Steed's saddle blink).
    EndOfCombat,
    /// "At the beginning of your next precombat main phase" (Chancellor of the
    /// Tangle's reveal), so the ritual mana isn't lost to a step change (CR
    /// 500.4).
    YourNextMainPhase,
    /// "At the beginning of your next main phase" — first or second,
    /// whichever of the controller's begins next (CR 505.1).
    YourNextMainPhaseAny,
    /// "At the beginning of the next main phase this turn" (Vivien's
    /// Stampede): as `YourNextMainPhaseAny`, but it lapses at this turn's
    /// cleanup when no main phase is left.
    NextMainPhaseThisTurn,
    /// "At the beginning of the next combat phase this turn" (Feral
    /// Encounter): as `NextCombat`, but it lapses at this turn's cleanup.
    NextCombatThisTurn,
    /// "At the beginning of the next cleanup step" (Waylay). Fires in the
    /// cleanup step of the turn it was registered in, so the objects it
    /// touches survive the end step.
    NextCleanupStep,
    /// "When you lose control of [target permanent], …" (Ray of Command): fires
    /// once, on the first control change away from the registering player.
    WhenYouLoseControlOfTarget,
    /// "When [this] leaves the battlefield or becomes untapped, …" (Merieke Ri
    /// Berit): watches the registering source; fires once, on whichever
    /// happens first.
    WhenSourceLeavesOrUntaps,
    /// "When you cast [target card] this turn, …" (Havengul Lich): fires once,
    /// on that card's `SpellCast` by the registering player; lapses at cleanup
    /// or when the card leaves its zone some other way (CR 400.7).
    WhenYouCastTargetThisTurn,
    /// "Whenever you attack this turn, …" (Jaya, Fiery Negotiator's −2): fires
    /// on each of the controller's attack declarations, not once, with the
    /// captured object still the body's `Target(0)`; lapses at cleanup.
    EachAttackThisTurn,
}

/// "If this is in your opening hand, you may …", resolved by
/// `GameState::apply_opening_hand_effects` after mulligans, before turn one.
///
/// * **Leyline / Gemstone Caverns** — the card begins the game on the
///   battlefield.
/// * **Chancellors** — the card stays in hand and is revealed to register a
///   one-shot trigger for later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpeningHandEffect {
    /// "If this is in your opening hand, you may begin the game with it on the
    /// battlefield", then `extra` (Gemstone Caverns' luck counter).
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
    /// "Any time you could mulligan, you may exile your hand and draw that
    /// many": an extra mulligan-decision answer, not run after mulligans; the
    /// variant flags the card so `apply_opening_hand_effects` skips it.
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

/// What an [`Effect::GoadWhile`] lasts for; resolves to a
/// `card::GoadHold` naming the source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoadLasts {
    /// "As long as this permanent remains on the battlefield" (Hot Pursuit).
    WhileSourceOnBattlefield,
    /// "For as long as that creature has a [kind] counter on it, it is
    /// goaded, can't attack you or a permanent you control, and can't block
    /// creatures you control" (Immortal Obligation).
    Obligation(crate::card::CounterType),
    /// "It's goaded for as long as they control it" (Vislor Turlough): held
    /// while the creature stays with the controller it had as this resolved.
    WhileControllerKeepsIt,
}

/// The one piece of resolver scratch an [`Effect::BindScratch`] pins.
/// Runtime-only: the suspend splices build these, no card does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScratchBinding {
    /// CR 701.38 — the seat whose ballot this `VoteTally::PerVote` run is for,
    /// read back by [`PlayerRef::CurrentVoter`].
    CurrentVoter(usize),
    /// CR 706.3a — the modified result this results-table arm was chosen by,
    /// read back by `Value::LastDieRoll`.
    LastDieRoll(u8),
    /// The opponent `Effect::ChooseOpponentThen` named, read back by
    /// [`PlayerRef::ChosenPlayerOfSource`] inside its body.
    ChosenOpponent(usize),
    /// The seat a body runs as (`MayDoBy`'s "that player may …"): the
    /// context's controller, not resolver scratch.
    Controller(usize),
    /// The amount a body reads as `Value::TriggerEventAmount` (what
    /// `MayPayGenericUpTo` was paid): the context's `event_amount`.
    EventAmount(u32),
    /// CR 122.6 — the player who puts the body's counters (the event's
    /// placer for "THEY put the same counters on this" — Bold Plagiarist),
    /// read back as each `CounterAdded`'s `placer`.
    Causer(usize),
    /// CR 603.7c — what the gating event left in the resolver's scratch (the
    /// sacrificed creature's power, the discarded card's mana value), carried
    /// onto the stack with a reflexive trigger that resolves later.
    Resolution(Box<ResolutionCarry>),
}

/// The resolution scratch a reflexive trigger's body may read, as the
/// enclosing effect left it ([`ScratchBinding::Resolution`]).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionCarry {
    pub sacrificed_power: Option<i32>,
    pub sacrificed_toughness: Option<i32>,
    pub sacrificed_mana_value: Option<u32>,
    pub sacrificed_total_power: i32,
    pub sacrificed_count: u32,
    pub last_discarded_mana_value: Option<u32>,
    pub greatest_discarded_mv: u32,
    pub last_exiled: Option<crate::card::CardId>,
    /// `Selector::LastMoved` — the card a reveal-until-find took (Yuna's Whistle).
    pub last_moved: Vec<crate::card::CardId>,
    /// `Selector::LastCreatedToken` — "create a token. When you do, attach
    /// it" (Dáin Ironfoot's Axe).
    #[serde(default)]
    pub last_created_token: Option<crate::card::CardId>,
    /// `Selector::AmassedArmy` — "amass. When you do, … the amassed Army's
    /// power" (Foray of Orcs, CR 701.47c).
    #[serde(default)]
    pub amassed_army: Option<crate::card::CardId>,
}

impl Default for Effect {
    /// `Noop`, so `CardDefinition` can derive `Default` and permanents without a
    /// spell effect use `..Default::default()`.
    fn default() -> Self { Effect::Noop }
}

mod query;

/// Serde default for `Effect::RollDie.modifier` — a zero (no-op) die-roll
/// modifier, so snapshots written before CR 706.2 modifiers existed still
/// deserialize cleanly.
pub fn zero_value() -> Value {
    Value::Const(0)
}

/// Serde default for `Effect::DiscardChosenFromRevealed.pick` — the one card
/// every printing before Noggin Whack chose.
pub fn one_value() -> Value {
    Value::ONE
}

/// What `Effect::LookTopExileOneMayPlay`'s exiled card may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LookExileGrant {
    /// Gonti: face down; cast it while it stays exiled, with mana of any type.
    #[default]
    AnyTypeWhileExiled,
    /// Vivien, Champion of the Wilds' −2: face down; cast it while it stays
    /// exiled if it's a creature spell, paying its cost.
    CreatureWhileExiled,
    /// Florian, Case of the Burning Masks: face up; play it this turn.
    PlayThisTurn,
    /// The Key to the Vault: a nonland card, cast now without paying its cost.
    CastFreeNonland,
    /// Durnan of the Yawning Portal: you MAY exile a creature card, face up;
    /// cast it while it stays exiled, paying its cost.
    CreatureMayWhileExiled,
    /// Djeru and Hazoret: you MAY exile a legendary creature card, face up;
    /// cast it this turn without paying its mana cost.
    LegendaryCreatureFreeThisTurn,
    /// Keldon Flamesage: you MAY exile an instant or sorcery card with mana
    /// value at most the look count, then cast it now without paying its cost.
    InstantSorceryMvAtMostCountFree,
    /// Feral Encounter: you MAY exile a creature card, face up; cast it this
    /// turn, paying its cost.
    CreatureMayThisTurn,
}

/// Serde default for `LookTopExileOneMayPlay.who` (Gonti's target opponent).
pub fn player_ref_target_zero() -> PlayerRef {
    PlayerRef::Target(0)
}

/// Serde default for `BindTargetSlot.count`.
pub fn one_u8() -> u8 {
    1
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
        | ZoneDest::Ante
        | ZoneDest::Command => false,
        ZoneDest::IfCard { then, else_, .. } => zonedest_has_target(then) || zonedest_has_target(else_),
        // The host is a slot another step declares ("that creature").
        ZoneDest::BattlefieldAttached { .. } => false,
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
    /// Printed "you MAY put … into your hand": a UI player's empty pick
    /// declines (all go to the rest-routing) and a short pick is respected.
    /// Mandatory picks (`false`) fill top-down; AutoDecider fills either way.
    #[serde(default)]
    pub optional: bool,
    /// "If you put a card into your hand this way, [effect]" — runs once
    /// after the picks land (Sidequest: Catch a Fish's Food + transform).
    #[serde(default)]
    pub then_if_picked: Option<Box<Effect>>,
    /// "If you didn't put a card into your hand this way, [effect]", once on the
    /// same `source` when the pick is empty (Rosecot Knight's counter).
    #[serde(default)]
    pub then_if_not_picked: Option<Box<Effect>>,
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
    /// "…and the rest on top of your library in any order" (Diabolic
    /// Vision): the non-picked cards stay on top, in revealed order.
    #[serde(default)]
    pub rest_on_top: bool,

    /// Picks matching this filter go onto the battlefield instead of to hand
    /// (Break Out's mana-value-2-or-less creature). Applied per pick, after
    /// `to_battlefield` and `picked_lands_to_battlefield`.
    #[serde(default)]
    pub picked_matching_to_battlefield: Option<SelectionRequirement>,
    /// Picks routed to the battlefield gain haste until end of turn
    /// (Break Out).
    #[serde(default)]
    pub battlefield_haste: bool,
    /// "A creature card and/or a land card" (Ojer Kaslem, In the Presence of
    /// Ages): each pick fills a distinct category, so `take` is at most one
    /// card per filter. Empty = no per-category limit.
    #[serde(default)]
    pub one_each: Vec<SelectionRequirement>,
    /// "Then if [predicate], put the rest into your hand" (Nine-Fingers
    /// Keene's nine Gates): read after the picks land; when it holds the
    /// rest go to hand (not a draw, CR 121.5) instead of their usual route.
    #[serde(default)]
    pub rest_to_hand_if: Option<Predicate>,
    /// "With total mana value N or less" (Michelangelo's Technique): a pick
    /// is kept only while the picks' mana values still sum to at most this.
    #[serde(default)]
    pub total_mana_value_cap: Option<Value>,
    /// The picks stay on top of the library instead of moving ("put up to two
    /// of them back on top … and the rest into your graveyard", Rowan's Grim
    /// Search); the rest follow the usual routing.
    #[serde(default)]
    pub keep_on_top: bool,
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
            then_if_not_picked: None,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
            rest_to_exile: false,
            rest_on_top: false,
            picked_matching_to_battlefield: None,
            battlefield_haste: false,
            one_each: Vec::new(),
            rest_to_hand_if: None,
            total_mana_value_cap: None,
            keep_on_top: false,
        }
    }
}

/// True when `effect` can emit a layer-4 card-type modification. Lives here
/// (not the engine) so `CardDefinition::type_scan_bits` can use it — it
/// depends only on the printed static. Twin of
/// [`static_effect_strips_abilities`]; both peel the gate wrappers because
/// `static_effect_to_effects` recurses through them.
pub fn static_effect_changes_card_types(effect: &StaticEffect) -> bool {
    use StaticEffect as SE;
    match effect {
        // `static_effect_to_effects` arms.
        SE::MatchingLandsAreCreatures { .. } | SE::AddCardTypeToMatching { .. } => true,
        // The off-battlefield half: no layer effect, but the same gate tells
        // the card-type readers a type may differ from the printed line.
        SE::OwnedCardsOffBattlefieldHaveCardType { .. } => true,
        // Stateful gather passes — each reads live GameState (devotion, a
        // counter count, a predicate), so it can't route through
        // `static_effect_to_effects`, but the printed static is the same tell.
        SE::NotCreatureWhileDevotionBelow { .. }
        | SE::NotCreatureUnless { .. }
        | SE::NonAuraEnchantmentsAreCreatures { .. }
        | SE::NoncreatureArtifactsAreCreatures
        | SE::SelfIsCreatureWhileCountersAtLeast { .. }
        | SE::SelfIsCreatureIf { .. } => true,
        SE::WhileClassLevelAtLeast { inner, .. }
        | SE::WhileYourTurn { inner }
        | SE::WhileNotYourTurn { inner }
        | SE::WhileCountersAtLeast { inner, .. }
        | SE::WhileCondition { inner, .. } => static_effect_changes_card_types(inner),
        _ => false,
    }
}

/// True when `effect` can emit a layer-4 land-type modification; used by
/// `CardDefinition::layer4_scan_bits`. Twin of
/// [`static_effect_changes_card_types`].
///
/// The list is exhaustive against the emitters in
/// `gather_continuous_effects_inner` and `static_effect_to_effects` (Alpine
/// Moon and Ultima count: they blank land types on the way to stripping).
/// `LandsYouControlAreChosenType` is a no-op until its ETB stamps
/// `chosen_land_type`, but the gate over-approximates: the static is the tell.
pub fn static_effect_changes_land_types(effect: &StaticEffect) -> bool {
    use StaticEffect as SE;
    match effect {
        SE::NamedLandsNeutralized
        | SE::BlightedLandsNeutralized
        | SE::GrantAllBasicLandTypes { .. }
        | SE::LandTypeChanger { .. }
        | SE::LandTypeChangerWhileCounters { .. }
        | SE::LandTypeChangerChosen { .. }
        | SE::LandsYouControlAreChosenType
        | SE::SelfIsChosenLandType => true,
        SE::WhileClassLevelAtLeast { inner, .. }
        | SE::WhileYourTurn { inner }
        | SE::WhileNotYourTurn { inner }
        | SE::WhileCountersAtLeast { inner, .. }
        | SE::WhileCondition { inner, .. } => static_effect_changes_land_types(inner),
        _ => false,
    }
}

/// Lives here for the same reason as [`static_effect_changes_land_types`].
/// True when `effect` can emit a layer-4 creature-type modification.
///
/// Exhaustive against the `AddCreatureType` / `SetCreatureTypes` emitters in
/// `static_effect_to_effects` and `gather_continuous_effects_inner`'s stateful
/// passes. `SelfIsCreatureIf` reads a live predicate, so it never routes
/// through `static_effect_to_effects`, but the static is the same tell.
pub fn static_effect_changes_creature_types(effect: &StaticEffect) -> bool {
    use StaticEffect as SE;
    match effect {
        SE::CreaturesYouControlAreChosenType
        | SE::MatchingAreChosenTypeToo { .. }
        | SE::MatchingLandsAreCreatures { .. }
        | SE::AddCreatureTypeToMatching { .. }
        | SE::EveryCreatureType { .. }
        | SE::MatchingLoseAllCreatureTypes { .. }
        | SE::SelfIsCreatureIf { .. }
        // Off-battlefield grants: no layer effect, but the lane is what tells
        // the hidden-zone evaluator to look (`off_battlefield_type_grants`).
        | SE::YourGraveyardCreaturesHaveChosenType
        | SE::OwnedCardsOffBattlefieldAreChosenTypeToo { .. }
        | SE::OwnedCardsOffBattlefieldAreTypeToo { .. }
        | SE::OwnedCardsOffBattlefieldAreEveryCreatureType { .. } => true,
        SE::WhileClassLevelAtLeast { inner, .. }
        | SE::WhileYourTurn { inner }
        | SE::WhileNotYourTurn { inner }
        | SE::WhileCountersAtLeast { inner, .. }
        | SE::WhileCondition { inner, .. } => static_effect_changes_creature_types(inner),
        _ => false,
    }
}

/// True when `effect` can put a layer-6 `Modification::RemoveAllAbilities`
/// into the gathered set — the static half of the gate on
/// `GameState::permanents_with_abilities_removed`. Over-approximates (state
/// gates ignored), so `true` means "gather and check". Every emitting gather
/// site `debug_assert!`s against this or `card_can_strip_abilities`, so a new
/// source can't bypass the gate.
///
/// Lives here so `CardDefinition::dispatch_scan_bits` can use it, as with
/// [`static_effect_changes_card_types`].
pub fn static_effect_strips_abilities(effect: &StaticEffect) -> bool {
    use StaticEffect as SE;
    match effect {
        // Dress Down; Titania's Song; Alpine Moon; Ultima.
        SE::CreaturesLoseAllAbilities
        | SE::MatchingLoseAllAbilities { .. }
        | SE::LandsLoseNonManaAbilities
        | SE::NoncreatureArtifactsLoseAbilities
        | SE::NamedLandsNeutralized
        | SE::BlightedLandsNeutralized => true,
        // Blood Moon replaces; Urborg only adds.
        SE::LandTypeChanger { replace, .. }
        | SE::LandTypeChangerWhileCounters { replace, .. }
        | SE::LandTypeChangerChosen { replace, .. } => {
            *replace
        }
        // Gate wrappers — `static_effect_to_effects` recurses through these
        // (and the stateful pass does for `WhileCondition`), so the predicate
        // must too.
        SE::WhileClassLevelAtLeast { inner, .. }
        | SE::WhileYourTurn { inner }
        | SE::WhileNotYourTurn { inner }
        | SE::WhileCountersAtLeast { inner, .. }
        | SE::WhileCondition { inner, .. } => static_effect_strips_abilities(inner),
        _ => false,
    }
}

/// True when `effect`, on a permanent other than the spell being cast, can
/// change what the spell costs — the presence predicate behind
/// `actions::CostStaticSources`.
///
/// ⚠ The list must be exactly the arms of the three source loops
/// (`extra_cost_for_spell_over`, `colored_spell_tax_for_spell_over`, and
/// `cost_reduction_for_spell_full_over`'s `for src in srcs`); a missing arm
/// silently misprices spells. The self-cost family (`SelfCostReduced*`,
/// `self_cost_reduction_if_control`, intrinsic affinity) is deliberately
/// absent: it is read off the card being cast, after the loop.
///
/// Audited by a `debug_assert_eq!` in `bot::can_afford_in_state_with` that
/// recomputes all three over the unfiltered board. Over-approximates like its
/// siblings: `true` means "keep the source and let the walk decide".
pub fn static_affects_spell_cost(effect: &StaticEffect) -> bool {
    use StaticEffect as SE;
    match effect {
        // `extra_cost_for_spell_over`
        SE::AdditionalCost { .. }
        | SE::AdditionalCostAfterFirstSpell { .. }
        | SE::NamedSpellTax { .. }
        | SE::EnchantedPlayerNamedSpellTax { .. }
        | SE::OpponentActivityCostsMoreOnYourTurn { .. }
        | SE::OpponentSpellsCostMore { .. }
        | SE::OpponentSpellsFromGraveyardOrExileCostMore { .. }
        | SE::SpellTaxPerControllerPermanent { .. }
        | SE::SpellsCostMoreExceptOnControllerTurn { .. }
        | SE::TaxOpponentSpellsTargeting { .. }
        | SE::TaxOpponentSpellsTargetingThis { .. }
        // Arachne's chosen-card-type tax was missing, so the bot's narrowed
        // source list dropped it and the cost-static audit fired in the
        // long pod test (Generous Gift, 2026-10-14).
        | SE::ChosenCardTypeSpellsCostMore { .. } => true,
        // `colored_spell_tax_for_spell_over`
        SE::ColoredSpellTax { .. } => true,
        // `cost_reduction_for_spell_full_over`'s source loop
        SE::AllPlayersSpellsCostLess { .. }
        | SE::AllPlayersCostReduction { .. }
        | SE::ChosenTypeSpellCostReduction { .. }
        | SE::ChosenColorsSpellCostReduction
        | SE::CostReduction { .. }
        | SE::CostReductionBySourcePower { .. }
        | SE::CostReductionByValue { .. }
        | SE::CostReductionPerTypeSharedWithExiled
        | SE::CostReductionDuringOpponentsTurn { .. }
        | SE::CostReductionFirstCreatureSpell { .. }
        | SE::CostReductionFirstInstantOrSorcery { .. }
        | SE::CostReductionFirstInstantOrSorceryPerValue { .. }
        | SE::CostReductionNthSpell { .. }
        | SE::CostReductionPerControllerExperience { .. }
        | SE::CostReductionPerCounterOnSource { .. }
        | SE::CostReductionTargetingFilter { .. }
        | SE::CostReductionTargetingHost { .. }
        | SE::CostReductionForYourSpellsTargetingThis { .. }
        | SE::CostReductionWhile { .. }
        | SE::ExileCastCostReduction { .. }
        | SE::LinkedExileCastCostReduction { .. }
        | SE::FirstMatchingSpellEachTurnCostsLess { .. }
        | SE::FirstKickedSpellEachTurnCostsLess { .. }
        // Source-side discounts the bot's affordability read (and its audit) must
        // also see: Zimone's per-counter first-X-spell, Family Matters' non-hand,
        // Grand Larceny's spells-you-don't-own.
        | SE::FirstMatchingSpellEachTurnCostsLessPerCounter { .. }
        | SE::NonHandCastCostReduction { .. }
        | SE::SpellsYouDontOwnCostLess { .. }
        | SE::GrantAffinityToISSpells { .. }
        | SE::GrantAffinityToSpells { .. }
        | SE::GraveyardCastCostReduction { .. }
        | SE::NamedSpellCostReduction { .. }
        | SE::SharedCreatureTypeSpellCostReduction { .. }
        | SE::YourISSpellsCostLessPerTargetCreature { .. }
        | SE::SpellsCostLessPerSameNameInYourGraveyard
        | SE::PerTargetSpellCost { .. } => true,
        // Gate wrappers: the reduction loop peels them all through `active_static`
        // (Class level, CR 717.2; Momo's "during your turn"); the tax walks peel
        // none. Over-approximating costs one iteration; dropping one misprices.
        SE::WhileClassLevelAtLeast { inner, .. }
        | SE::WhileYourTurn { inner }
        | SE::WhileNotYourTurn { inner }
        | SE::WhileCountersAtLeast { inner, .. }
        | SE::WhileCondition { inner, .. } => static_affects_spell_cost(inner),
        _ => false,
    }
}

/// True when `effect` is a `SelfCostReduced*` static that
/// `cost_reduction_for_spell_full_over`'s tail reads off the card being cast
/// (cf. [`static_affects_spell_cost`] for board sources). The tail
/// `debug_assert_eq!`s against this per static, so the lists can't drift;
/// [`CardDefinition::has_self_cost_reduction`] is the memoized gate.
pub fn static_effect_is_self_cost_reduction(effect: &StaticEffect) -> bool {
    use StaticEffect as SE;
    matches!(
        effect,
        SE::SelfCostReducedByGreatestPower
            | SE::SelfCostReducedByTotalPower
            | SE::SelfCostReducedPerCreatureInGraveyard
            | SE::SelfCostReducedPerCardTypeInGraveyard
            | SE::SelfCostReducedByNoncreatureArtifactMv
            | SE::SelfCostReducedByDistinctLandNames
            | SE::SelfCostReducedPerGraveyardCardMatching { .. }
            | SE::SelfCostReducedPerPermanentMatching { .. }
            | SE::SelfCostReducedIfCreatureDiedThisTurn { .. }
            | SE::SelfCostReducedIfPredicate { .. }
            | SE::SelfCostReducedByDomain { .. }
            | SE::SelfCostReducedDuringYourTurn { .. }
            | SE::SelfCostReducedByDevotion { .. }
            | SE::SelfCostReducedIfControlEach { .. }
            | SE::SelfCostReducedIf { .. }
            | SE::SelfCostReducedByValue { .. }
            | SE::SelfCostReducedPerDiscardThisTurn { .. }
            | SE::SelfCostReducedPerOpponent { .. }
            | SE::SelfCostReducedPerSpellCastThisTurn { .. }
            | SE::SelfCostReducedPerCreatureAttackedThisTurn { .. }
    )
}

/// The `GrantActivatedAbility` twin of [`static_grants_triggered_ability`],
/// same wrappers and over-approximation: `grant_scan` peels and evaluates the
/// gates, so only `false` is acted on.
pub fn static_effect_grants_activated(effect: &StaticEffect) -> bool {
    use StaticEffect as SE;
    match effect {
        SE::GrantActivatedAbility { .. }
        | SE::ControlledHaveAbilitiesOfExiledWithSource { .. }
        | SE::ChosenPermanentAbilitiesCantBeActivated => true,
        SE::WhileClassLevelAtLeast { inner, .. }
        | SE::WhileYourTurn { inner }
        | SE::WhileNotYourTurn { inner }
        | SE::WhileCountersAtLeast { inner, .. }
        | SE::WhileCondition { inner, .. } => static_effect_grants_activated(inner),
        _ => false,
    }
}

/// True when `effect` can be a `GrantTriggeredAbility` once the gate wrappers
/// are peeled, i.e. when the engine's `active_static` could return one. Uses
/// `active_static`'s wrapper set, so edit the two together.
pub fn static_grants_triggered_ability(effect: &StaticEffect) -> bool {
    use StaticEffect as SE;
    match effect {
        SE::GrantTriggeredAbility { .. } => true,
        SE::WhileClassLevelAtLeast { inner, .. }
        | SE::WhileYourTurn { inner }
        | SE::WhileNotYourTurn { inner }
        | SE::WhileCountersAtLeast { inner, .. }
        | SE::WhileCondition { inner, .. } => static_grants_triggered_ability(inner),
        _ => false,
    }
}

/// True when `effect` can emit a layer-6 `AddKeyword` matching `pred`. Twin of
/// [`static_effect_strips_abilities`] for keyword grants.
///
/// Scanning the enum for `Keyword` fields is necessary but not sufficient:
/// some such variants only remove a keyword (`LoseKeyword`,
/// `CantHaveKeyword`), and some grant one without carrying the field — those
/// are spelled out below.
///
/// Lives here so [`CardDefinition::can_grant_keyword`] can use it, as with
/// [`static_effect_changes_card_types`].
///
/// [`CardDefinition::can_grant_keyword`]: crate::card::CardDefinition::can_grant_keyword
pub fn static_effect_grants_keyword(
    effect: &StaticEffect,
    pred: &impl Fn(&Keyword) -> bool,
) -> bool {
    use StaticEffect as SE;
    let any = |kws: &[Keyword]| kws.iter().any(pred);
    match effect {
        SE::PumpSelfIf { keywords, .. }
        | SE::PumpTeamIf { keywords, .. }
        | SE::GrantPumpSelfIf { keywords, .. }
        | SE::MatchingLandsAreCreatures { keywords, .. }
        | SE::AnthemForFilter { keywords, .. }
        | SE::AnthemForFilterIf { keywords, .. } => any(keywords),
        SE::GrantKeyword { keyword, .. }
        | SE::GrantKeywordWhileControllerControlsAtMost { keyword, .. }
        | SE::GrantKeywordToChosenType { keyword, .. }
        | SE::GrantKeywordToAttackers { keyword }
        | SE::GraveyardAnthem { keyword, .. }
        | SE::SelfHasKeywordWhile { keyword, .. }
        | SE::SelfHasKeywordWhilePredicate { keyword, .. }
        | SE::SelfHasKeywordWhileCountersAtLeast { keyword, .. }
        | SE::SelfHasKeywordIf { keyword, .. } => pred(keyword),
        // Layer 4, but it reads as a computed changeling (`layers.rs`).
        SE::EveryCreatureType { .. } => pred(&Keyword::Changeling),
        // Unbounded: the granted keyword comes from a payload the static doesn't
        // fix (exiled cards, a draft note, a chosen colour or type, a counter
        // count), so the only sound answer is "maybe"; all are rare enough that
        // the gate still reads `false` on nearly every board. Swirl the Mists
        // (`AllColorWordsBecomeChosen`, CR 612) is here too: a text change turns
        // `Protection(A)` into `Protection(B)`. `static_effect_removes_keyword`
        // carries it for the other direction.
        SE::GainKeywordsOfCards { .. }
        | SE::SelfHasDraftNotedKeywords
        | SE::AnnihilatorPerPlusOneCounter
        | SE::GrantProtectionFromChosenColor { .. }
        | SE::GrantProtectionFromChosenCreatureType { .. }
        | SE::ProtectionFromExiledWithCardTypes
        | SE::AllColorWordsBecomeChosen
        | SE::YouAndCreaturesProtectionFromChosenCardType => true,
        SE::WhileClassLevelAtLeast { inner, .. }
        | SE::WhileYourTurn { inner }
        | SE::WhileNotYourTurn { inner }
        | SE::WhileCountersAtLeast { inner, .. }
        | SE::WhileCondition { inner, .. } => static_effect_grants_keyword(inner, pred),
        _ => false,
    }
}

/// What an activation's costs did, carried on the stack by
/// `Effect::WithPaidCosts` (CR 602.2b — costs are paid as the ability is
/// activated, not as it resolves).
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct PaidCostRecord {
    /// `Selector::CostExiledCards`.
    pub exiled_cards: Vec<crate::card::CardId>,
    /// `Value::ExiledForCostManaValue`.
    pub exiled_mana_value: Option<i32>,
    /// `R::SharesCreatureTypeWithTapped`.
    pub tapped: Vec<crate::card::CardId>,
    /// `Effect::ExileCostSacrificedBatch`.
    pub sacrificed_batch: Vec<crate::card::CardId>,
    /// `Value::LastDiscardedManaValue`'s cost fallback.
    pub discarded_mana_value: Option<u32>,
    /// `Selector::ExiledForCost`.
    #[serde(default)]
    pub exiled_card: Option<crate::card::CardId>,
}

/// What `Effect::WithSacrificedPt` re-stamps beside the numbers: the
/// sacrificed permanent's type and colour flags at the time it was paid.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct SacrificedTraits {
    pub artifact: Option<bool>,
    pub outlaw: Option<bool>,
    pub vehicle: Option<bool>,
    pub colors: Option<Vec<crate::mana::Color>>,
}
