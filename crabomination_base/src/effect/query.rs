//! Query / targeting methods on [`Effect`]: target requirements, per-slot
//! target filters, friendly/graveyard target hints, and short-text
//! rendering. Split out of `effect.rs` (no behavior change).

use super::*;

/// Implicit creature restriction for a bare, unfiltered target on a
/// creature-only pump effect. A `Selector::Target(n)` carries no
/// `SelectionRequirement`, but you can't give +3/+3 to (or set the base P/T
/// of) a land — the target must be a creature. Surfacing this filter makes
/// cast-time legality and the auto-targeter reject non-creatures.
/// (`TargetFiltered` selectors already carry their own, stricter, filter;
/// `BecomeCreature` deliberately targets *non*-creatures and is excluded.)
static IMPLICIT_CREATURE_TARGET: SelectionRequirement = SelectionRequirement::Creature;

/// Earthbend (CR 701.66a) targets "a land you control" — surfaced for the
/// auto-targeter / cast-time legality. Built lazily since `.and()` boxes.
static EARTHBEND_TARGET: std::sync::LazyLock<SelectionRequirement> =
    std::sync::LazyLock::new(|| {
        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou)
    });

/// Finale of Promise — slot 0 (instant) / slot 1 (sorcery), each a graveyard
/// card of mana value X or less. Built lazily since `.and()` boxes.
static FINALE_INSTANT_SLOT: std::sync::LazyLock<SelectionRequirement> =
    std::sync::LazyLock::new(|| {
        SelectionRequirement::HasCardType(crate::card::CardType::Instant)
            .and(SelectionRequirement::InYourGraveyard)
            .and(SelectionRequirement::ManaValueAtMostXFromCost)
    });
static FINALE_SORCERY_SLOT: std::sync::LazyLock<SelectionRequirement> =
    std::sync::LazyLock::new(|| {
        SelectionRequirement::HasCardType(crate::card::CardType::Sorcery)
            .and(SelectionRequirement::InYourGraveyard)
            .and(SelectionRequirement::ManaValueAtMostXFromCost)
    });

/// Player restriction synthesized for the player slot referenced by a
/// `Selector::ControlledBy { who: PlayerRef::Target(n) }` — the spell targets
/// a player and then acts on the permanents that player controls (Sleep).
static IMPLICIT_PLAYER_TARGET: SelectionRequirement = SelectionRequirement::Player;

/// `Some(&Player)` when `what` is `ControlledBy { who: Target(n) }` for `slot`.
fn implicit_player_for_slot(what: &Selector, slot: u8) -> Option<&'static SelectionRequirement> {
    matches!(what, Selector::ControlledBy { who: PlayerRef::Target(n), .. } if *n == slot)
        .then_some(&IMPLICIT_PLAYER_TARGET)
}

/// Slot-agnostic sibling of [`implicit_player_for_slot`] for the primary
/// target filter ("tap all lands target player controls" — Mistbind Clique).
fn implicit_player_if_controlled_by_target(
    what: &Selector,
) -> Option<&'static SelectionRequirement> {
    matches!(what, Selector::ControlledBy { who: PlayerRef::Target(_), .. })
        .then_some(&IMPLICIT_PLAYER_TARGET)
}

/// `Some(&Player)` when `what` is a bare `Selector::Player(Target(_))`
/// (Oona's "target opponent exiles…").
fn implicit_player_if_bare_player_target(
    what: &Selector,
) -> Option<&'static SelectionRequirement> {
    matches!(what, Selector::Player(PlayerRef::Target(_))).then_some(&IMPLICIT_PLAYER_TARGET)
}

/// Slot-keyed sibling of [`implicit_player_if_bare_player_target`].
fn implicit_player_for_bare_player_slot(
    what: &Selector,
    slot: u8,
) -> Option<&'static SelectionRequirement> {
    matches!(what, Selector::Player(PlayerRef::Target(n)) if *n == slot)
        .then_some(&IMPLICIT_PLAYER_TARGET)
}

/// `Some(&Player)` when a bare `PlayerRef::Target(n)` fills `slot` — effects
/// that target a player directly through a `PlayerRef` field
/// (`ExilePlayerGraveyard`, `ExileHand`, `DiscardUnlessKind`).
fn implicit_player_for_ref_slot(who: &PlayerRef, slot: u8) -> Option<&'static SelectionRequirement> {
    matches!(who, PlayerRef::Target(n) if *n == slot).then_some(&IMPLICIT_PLAYER_TARGET)
}

/// `Some(&Creature)` when `what` is any bare numbered target (slot-agnostic —
/// used for the "primary" target filter).
fn implicit_creature_if_bare_target(what: &Selector) -> Option<&'static SelectionRequirement> {
    matches!(what, Selector::Target(_)).then_some(&IMPLICIT_CREATURE_TARGET)
}

/// `Some(&Creature)` when `what` is the bare numbered target for `slot`.
fn implicit_creature_for_slot(what: &Selector, slot: u8) -> Option<&'static SelectionRequirement> {
    matches!(what, Selector::Target(n) if *n == slot).then_some(&IMPLICIT_CREATURE_TARGET)
}

impl Effect {
    pub const NOOP: Effect = Effect::Noop;

    /// True if this effect is an *adapt* effect (CR 702.108) — the
    /// counter-check shape produced by `shortcut::adapt`: "if this creature
    /// has no +1/+1 counters, put N +1/+1 counters on it." Recognizes the
    /// bare effect and the same shape as the head of a `Seq` (adapt cards that
    /// bundle a rider). Used to flag `AbilityActivated` events as adapt-ability
    /// activations without tagging every adapt card by hand.
    pub fn is_adapt(&self) -> bool {
        use crate::card::{CounterType, SelectionRequirement};
        match self {
            Effect::If { cond, then, .. } => {
                let counter_check = matches!(
                    cond,
                    Predicate::Not(inner) if matches!(
                        &**inner,
                        Predicate::EntityMatches {
                            what: Selector::This,
                            filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                        }
                    )
                );
                let adds_counter = matches!(
                    &**then,
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, .. }
                );
                counter_check && adds_counter
            }
            Effect::Seq(effects) => effects.first().is_some_and(Effect::is_adapt),
            _ => false,
        }
    }

    pub fn seq(effects: Vec<Effect>) -> Self {
        if effects.is_empty() { Effect::Noop }
        else if effects.len() == 1 { effects.into_iter().next().unwrap() }
        else { Effect::Seq(effects) }
    }

    /// True if this effect (transitively) requires a chosen target (i.e.
    /// references `Selector::Target(_)` anywhere). Used for cast-time
    /// validation.
    pub fn requires_target(&self) -> bool {
        fn sel_has_target(s: &Selector) -> bool {
            match s {
                Selector::Target(_) | Selector::TargetFiltered { .. } => true,
                Selector::AttachedTo(i)
                | Selector::AttachedToMe(i)
                | Selector::RadianceGroup { subject: i }
                | Selector::CreaturesInCombatWith(i)
                | Selector::SharingNameWith(i) => sel_has_target(i),
                Selector::Take { inner, count }
                | Selector::TakeRandom { inner, count } => {
                    sel_has_target(inner) || value_has_target(count)
                }
                Selector::TakeWithSumCap { inner, cap, value_of_each } => {
                    sel_has_target(inner)
                        || value_has_target(cap)
                        || value_has_target(value_of_each)
                }
                Selector::TopOfLibrary { who, .. }
                | Selector::BottomOfLibrary { who, .. }
                | Selector::CardsInZone { who, .. }
                | Selector::ControlledBy { who, .. }
                | Selector::Player(who) => player_has_target(who),
                _ => false,
            }
        }
        fn player_has_target(p: &PlayerRef) -> bool {
            match p {
                PlayerRef::Target(_) => true,
                PlayerRef::OwnerOf(s) | PlayerRef::ControllerOf(s) => sel_has_target(s),
                _ => false,
            }
        }
        fn value_has_target(v: &Value) -> bool {
            match v {
                Value::CountOf(s) | Value::PowerOf(s) | Value::ToughnessOf(s)
                | Value::MarkedDamageOn(s) => sel_has_target(s),
                Value::CountersOn { what, .. } => sel_has_target(what),
                Value::LifeOf(p) | Value::HandSizeOf(p) | Value::GraveyardSizeOf(p)
                | Value::LibrarySizeOf(p) | Value::PlayerSpeed(p)
                | Value::PermanentCountControlledBy(p)
                | Value::CreatureCountControlledBy(p) => {
                    player_has_target(p)
                }
                Value::Sum(vs) => vs.iter().any(value_has_target),
                Value::Diff(a, b) | Value::Times(a, b) | Value::Min(a, b) | Value::Max(a, b) => {
                    value_has_target(a) || value_has_target(b)
                }
                Value::NonNeg(v) => value_has_target(v),
                Value::ManaValueOf(s) => sel_has_target(s),
                Value::ColorCountOf(s) => sel_has_target(s),
                Value::LoyaltyOf(s) => sel_has_target(s),
                _ => false,
            }
        }
        fn pred_has_target(p: &Predicate) -> bool {
            match p {
                Predicate::Not(q) => pred_has_target(q),
                Predicate::All(v) | Predicate::Any(v) => v.iter().any(pred_has_target),
                Predicate::SelectorExists(s) => sel_has_target(s),
                Predicate::SelectorCountAtLeast { sel, n } => sel_has_target(sel) || value_has_target(n),
                Predicate::ValueAtLeast(a, b)
                | Predicate::ValueAtMost(a, b)
                | Predicate::ValueEquals(a, b) => value_has_target(a) || value_has_target(b),
                Predicate::IsTurnOf(p) => player_has_target(p),
                Predicate::EntityMatches { what, .. } => sel_has_target(what),
                _ => false,
            }
        }
        match self {
            Effect::Noop
            | Effect::ExileAnyNumberUntilSourceLeaves { .. }
            | Effect::RevealUntilCreatureDoubleBasePt
            | Effect::CopyActivatedAbilityMayChooseTargets
            | Effect::AdvanceClassLevel
            | Effect::SignalTheClans
            | Effect::NivMizzetReveal
            | Effect::MarkExileReturnOnResolve { .. }
            | Effect::EachOpponentExilesHandCardOrPermanent
            | Effect::EachOpponentWithoutLegendaryLoses
            | Effect::UnexpectedResults
            | Effect::PayLifeRevealExileFromHand { .. }
            | Effect::ChannelLifeForMana
            | Effect::CantLoseThisTurn { .. }
            | Effect::Venture
            | Effect::DoubleYourSourcesDamageThisTurn
            | Effect::ReturnSelfTransformedAttached
            | Effect::SecondSunrise
            | Effect::PlayerTapsUntapped { .. }
            | Effect::TapAnyNumberThenPumpPerTapped { .. }
            | Effect::GrantExtraPlusOneCountersThisTurn { .. }
            // Amount is a scratch read (CounteredSpellManaSpent), no slots.
            | Effect::AddManaAtNextMainPhase { .. }
            // Free-cast offers pick their own targets at cast time.
            | Effect::CastAnyOrderWithoutPaying { .. }
            // A reflexive sub-trigger's body picks its targets when it's
            // pushed (CR 603.7d), not at the containing cast/activation.
            | Effect::ReflexiveTrigger { .. }
            | Effect::PutResolvingSpellInLibraryFromTop(_)
            // Group / each-player effects with no cast-time target slot.
            | Effect::EachPlayerPutsHandCardOnTop { .. }
            | Effect::LandsBecomeChosenBasicType { .. }
            | Effect::ChooseBasicLandTypeForSource
            | Effect::ExileTopSelfPumpIfCreature
            | Effect::ChooseRandomGraveyardCardCreatureToBattlefieldElseHand { .. }
            | Effect::DistributeCountersAmongLastCreated { .. } => false,
            // Mills the controller's own library, then branches on the milled
            // card's type into token-minting sub-effects — no cast-time target.
            Effect::MillThenBranchByType { .. } => false,
            // "As this enters, choose a number" — no cast-time target.
            Effect::ChooseNumberForSource { .. } => false,
            // "Choose a number; destroy all creatures with power ≥ it" — the
            // number is picked at resolution; the wipe is untargeted.
            Effect::ChooseNumberDestroyByPower { .. } => false,
            // "As this enters, choose a permanent" — chosen at resolution.
            Effect::ChoosePermanentForSource { .. } => false,
            // CR 603.7 — a reflexive payoff is opaque to cast-time target
            // validation; its body's targets are chosen when it resolves.
            Effect::Reflexive { .. } => false,
            // "You may pay {X}" — the paid amount and body resolve later.
            Effect::MayPayGenericUpTo { .. } => false,
            // CR 701.54 — untargeted; the Ring-bearer is chosen at resolution.
            Effect::RingTempts { .. } => false,
            Effect::SacrificeAtEndOfCombat { .. } => false,
            // CR 603.7e — registers a player-scoped rider; no cast-time target.
            Effect::GrantNextCreatureSpellCounters { .. } => false,
            Effect::GrantNextCreatureSpellKeyword { .. } => false,
            // Removes counters from your creatures and doubles onto self — untargeted.
            Effect::DoubleP1P1CountersFromYourCreatures => false,
            // Locks opponents' weak creatures out of blocking — untargeted.
            Effect::OpponentWeakCreaturesCantBlockByYourCounters => false,
            // Symmetric discard-and-make-tokens — untargeted.
            Effect::EachPlayerDiscardsHandMakeTokens { .. } => false,
            // Coin-flip board wipe — untargeted.
            Effect::CoinFlipEachCreatureDestroyOnTails { .. } => false,
            // Installs a player-scoped attack tax; no cast-time target.
            Effect::TaxAttackersUntilYourNextTurn { .. } => false,
            // Player-scoped ETB-counter grant; no cast-time target.
            Effect::CreaturesEnterWithExtraCounterThisTurn { .. } => false,
            // Random graveyard pick at resolution — no cast-time target.
            Effect::ExileRandomGraveyardCopyTapped { .. } => false,
            // Registers a floating trigger; no cast-time target.
            Effect::OnMatchingAttacksThisTurn { .. } => false,
            Effect::CopyAbility { what, .. } => sel_has_target(what),
            Effect::StaggerPlayerUntilYourNextTurn { who } => player_has_target(who),
            Effect::LookTopKeepOneRestToGraveyard { who, .. } => {
                who.as_ref().is_some_and(player_has_target)
            }
            Effect::LookTopPutMatchingOntoBattlefield { .. } => false,
            Effect::MillDeployCreaturesUntilEndStep { .. } => false,
            Effect::ExileEachTopFreePlayLesser => false,
            Effect::LookTopTakeOneDeployLandsRestGraveyard { .. } => false,
            Effect::ReduceEquipCost { .. } | Effect::SacrificeAtNextUpkeep { .. } => false,
            Effect::Unattach { what } => sel_has_target(what),
            Effect::SetSaddled { what } => sel_has_target(what),
            Effect::AtNextEndStep { body } => body.requires_target(),
            Effect::RevealFiveDraftAgainstOpponent => false,
            Effect::EncoreTokens => false,
            // Targets an opponent, but resolution auto-binds slot 0 / the
            // lowest-seat opponent, so no cast-time target is demanded.
            Effect::RevealOpponentTopPutOntoBattlefield { .. } => false,
            Effect::NameCardRevealTop { .. } => false,
            Effect::RevealTopToHandLoseMv { .. } => false,
            Effect::PutFromHandOrGraveyardOntoBattlefield { .. } => false,
            Effect::ReturnExiledBySourceToBattlefield { .. } => false,
            Effect::StealCreatureEtbThisTurn => false,
            // Sundering Titan auto-picks one land per basic type at resolution.
            Effect::DestroyLandOfEachBasicType => false,
            // Untargeted mass destroy keyed on a mana-value count.
            Effect::DestroyEachNonlandWithManaValue { .. } => false,
            Effect::DestroyEachCreatureWithManaValue { .. } => false,
            Effect::AttackDespiteDefenderThisTurn { .. } => false,
            Effect::LookTopExileOneMayPlay { .. } => false,
            Effect::LookTopDeployLandOrHand { .. } => false,
            Effect::LookTopMayDeployLand { .. } => false,
            // Targets are chosen at resolution (Decision::ChooseCards), so no
            // cast-time target slot is demanded.
            Effect::TapUpToValue { .. } => false,
            // CR 702.55 — the haunted creature is auto-picked at resolution.
            Effect::HauntCreature { .. } => false,
            // CR 701.31 — voting is untargeted; choices happen at resolution.
            Effect::WillOfTheCouncilExile { .. } => false,
            Effect::CycleRecurFromGraveyard { .. } => false,
            Effect::ReturnGraveyardPermanentsDifferentNames => false,
            Effect::ReturnGraveyardCardsToHand { .. } => false,
            // Resolution-time ChooseCards by the affected player; untargeted.
            // A `who: PlayerRef::Target(n)` makes the affected player a
            // real cast-time target (Quandrix Command mode 3).
            Effect::ShuffleGraveyardCardsIntoLibrary { who, .. } => player_has_target(who),
            Effect::LookTopNDeployPermanentsRestToHand { .. } => false,
            Effect::LookTopMayDeployAttacking { .. } => false,
            Effect::ExileTopUntilPermanentToBattlefieldOrHand => false,
            Effect::ExileTopUntilNonlandMayPlay { .. } => false,
            Effect::ReturnGraveyardCreaturesUpToTotalPower { .. } => false,
            Effect::ReturnGraveyardCreaturesUpToTotalManaValue { .. } => false,
            Effect::CommandTheDreadhorde => false,
            Effect::LookTopMayRevealMatchToHandElseBottom { .. } => false,
            Effect::NameCardTargetDiscardsMatching
            | Effect::NameCardExileMatchingAllZones
            | Effect::FertileImagination { .. }
            | Effect::GuildFeud
            | Effect::NameCardTargetDiscardsOneOrYouDraw => true,
            Effect::ChooseTypeRevealTopPartition { .. } => false,
            Effect::AethermagesTouch { .. } => false,
            Effect::InfernalTutor => false,
            Effect::IgnorantBliss => false,
            Effect::Dovescape => false,
            Effect::IsperiaReveal => false,
            Effect::KindleTheCarnage => false,
            Effect::ChooseTwoColorsForSource | Effect::GainLifePerChosenColorOfCast => false,
            Effect::GraveBetrayalRegister | Effect::GraveBetrayalReanimate => false,
            Effect::TemptingOffer { body } => body.requires_target(),
            // The accept branch's slot-0 player is bound at resolution; only
            // `otherwise` can demand a cast-time target (Browbeat's drawer).
            Effect::PlayersMayAccept { otherwise, .. } => otherwise.requires_target(),
            Effect::OnEachSpellCastThisTurn { .. } => false,
            Effect::PutExiledCreatureOntoBattlefield { .. } => false,
            Effect::ExileHand { who } => player_has_target(who),
            Effect::ExileChosenFromHandOrGraveyard { who, .. } => player_has_target(who),
            Effect::DiscardUnlessKind { who, count, .. } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::RevealTopToHandLoseLifeRepeat => false,
            Effect::Demonstrate => false,
            Effect::Cipher => false,
            Effect::Myriad => false,
            Effect::JoinCombatAttacking { what } => sel_has_target(what),
            Effect::Enlist => false,
            Effect::StudyTopCard { .. } => false,
            Effect::ExileTopWithCounters { .. } => false,
            Effect::GrantPlayFromTopThisTurn => false,
            Effect::HoneFromHand { .. } => false,
            Effect::PutFromHandOntoBattlefield { .. } => false,
            Effect::DeployCreatureFromHandAttacking { .. } => false,
            Effect::LockCreatureAndPlaneswalkerCasts => false,
            Effect::ExileTopFaceDownTokenReturns { .. } => false,
            Effect::DeployLandsFromHandAndGraveyard { .. } => false,
            Effect::Manifest { .. } => false,
            Effect::ManifestFromHand { who, count, .. } => {
                sel_has_target(who) || value_has_target(count)
            }
            Effect::WishToHand { .. } => false,
            Effect::SacrificeAllButOnePerType { who } => sel_has_target(who),
            Effect::EachPlayerKeepsOneSacrificeRest { who, .. } => sel_has_target(who),
            Effect::RevealRandomDiscardNonland { who, .. } => sel_has_target(who),
            // Search-library / counter-all effects pick no cast-time target.
            Effect::SearchLibraryCreaturesUpToTotalManaValue { .. }
            | Effect::CounterAllOtherSpellsDrawPer => false,
            Effect::DestroyTargetsPolymorph { .. } => true,
            Effect::DestroyTargets { .. } => true,
            Effect::DealHalfLifeDamage { .. } => false,
            Effect::Champion { .. } => false,
            Effect::ExileUpToNFromGraveyards { count, .. } => value_has_target(count),
            Effect::SpellTaxUntilYourNextTurn { .. } => false,
            Effect::CreateTokenAttachedTo { target, .. } => sel_has_target(target),
            Effect::CreateTokenAttachedToEach { target, .. } => sel_has_target(target),
            Effect::ManifestDread { .. } => false,
            Effect::ManifestDreadRepeatThenCounters { .. } => false,
            Effect::Cloak { .. } => false,
            Effect::CatchUpBasicLands => false,
            Effect::ExileUntilDuplicateName { .. } => false,
            Effect::ExileFromHandTaxed { .. } => false,
            Effect::Hideaway { .. } => false,
            Effect::NthResolutionThisTurn { branches } => {
                branches.iter().any(|e| e.requires_target())
            }
            Effect::SacrificeSource => false,
            Effect::ExileSource => false,
            Effect::SacrificeSourceUnlessSacrifice { .. } => false,
            Effect::GrantNextInstantOrSorceryDiscountThisTurn { .. } => false,
            Effect::ReturnSelfAsEnchantment => false,
            Effect::ReturnSelfTappedWithCounters { .. } => false,
            Effect::ReturnSelfTapped => false,
            Effect::ReturnSelf => false,
            Effect::ExileAndReturnSelfWithSaddler => false,
            Effect::ReturnTopCreatureFromGraveyard { .. } => false,
            Effect::Transform { what } => sel_has_target(what),
            Effect::BecomeRenowned { what } => sel_has_target(what),
            Effect::Flip { what } => sel_has_target(what),
            Effect::Meld { .. } => false,
            Effect::SpellsCostLessThisTurn { .. } => false,
            Effect::CastFromHandWithoutPaying { .. } => false,
            Effect::PreventNextDamageFromChosenSource { .. } => false,
            Effect::RevealTopPayOrTake { .. } => false,
            Effect::DigForLandToBattlefield { .. } => false,
            Effect::Tribute { otherwise, .. } => otherwise.requires_target(),
            Effect::Seq(v) => v.iter().any(|e| e.requires_target()),
            Effect::If { cond, then, else_ } => {
                pred_has_target(cond) || then.requires_target() || else_.requires_target()
            }
            Effect::ForEach { selector, body } => {
                sel_has_target(selector) || body.requires_target()
            }
            Effect::Repeat { count, body } => value_has_target(count) || body.requires_target(),
            Effect::FlipCoin { count, on_heads, on_tails } => {
                value_has_target(count)
                    || on_heads.requires_target()
                    || on_tails.requires_target()
            }
            Effect::ManaClash { opponent } => sel_has_target(opponent),
            Effect::FlipCoinsUntilLoseOrStop { tiers } => {
                tiers.iter().any(|(_, e)| e.requires_target())
            }
            Effect::FlipCoinsChooseCount { per_win, per_loss, all_won, .. } => {
                per_win.requires_target()
                    || per_loss.requires_target()
                    || all_won.requires_target()
            }
            Effect::MoveCounters { from, to, amount, .. } => {
                sel_has_target(from) || sel_has_target(to) || value_has_target(amount)
            }
            Effect::FreeSpellsFromHandThisTurn => false,
            Effect::ChooseCardTypeForSource => false,
            Effect::PlayFromGraveyardThisTurn
            | Effect::ExileYourGraveyardBoundThisTurn
            | Effect::GlimpseOfTomorrow
            | Effect::GarthOneEye { .. }
            | Effect::GristPlusOne => false,
            // Targets a spell on the stack.
            Effect::ChefsKiss => true,
            Effect::AdjustBattleDefense { what } => sel_has_target(what),
            // Targets an opponent (player slot 0).
            Effect::OpponentRevealsPickToBattlefield { .. } => true,
            Effect::RollDie { count, results, .. } => {
                value_has_target(count) || results.iter().any(|(_, _, e)| e.requires_target())
            }
            Effect::ChooseMode(modes) => modes.iter().any(|e| e.requires_target()),
            Effect::ChooseN { modes, .. } => modes.iter().any(|e| e.requires_target()),
            // ChooseUpToN's modes are self-targeting (chosen at resolution).
            Effect::ChooseUpToN { .. } => false,
            Effect::Escalate { modes, .. } => modes.iter().any(|e| e.requires_target()),
            // Spree targets are supplied per chosen mode at cast time and
            // consumed at resolution; no fixed cast-time slot is demanded.
            Effect::Spree { .. } | Effect::Tiered { .. } | Effect::ChooseModesCast { .. } => false,
            Effect::MayDo { body, .. } | Effect::CapTargetsAtX { body } => body.requires_target(),
            Effect::MayPayX { body, .. } => body.requires_target(),
            Effect::OptionalTargets { body, .. } => body.requires_target(),
            Effect::WithSacrificedPt { body, .. } => body.requires_target(),
            Effect::WithTappedPower { body, .. } => body.requires_target(),
            Effect::OnYourNextSpellCastThisTurn { body }
            | Effect::OnYourNextInstantSorceryThisTurn { body }
            | Effect::OnYourNextNamedSpellThisTurn { body } => body.requires_target(),
            Effect::SearchSplitWithOpponent { .. } => false,
            Effect::FactOrFiction { .. } => false,
            Effect::ReanimateAurasExileEot => false,
            Effect::AllureOfTheUnknown => false,
            Effect::PossibilityStorm => false,
            Effect::ReturnResolvingSpellToHand => false,
            Effect::ExileResolvingSpell => false,
            Effect::SilencePlayersThisTurn { who } => player_has_target(who),
            Effect::MayPay { body, .. } | Effect::MayPayLife { body, .. } => body.requires_target(),
            Effect::MaySacrifice { then, else_, .. }
            | Effect::MaySacrificeSource { then, else_, .. }
            | Effect::MayTap { then, else_, .. }
            | Effect::MayDiscard { then, else_, .. }
            | Effect::MayDiscardMatching { then, else_, .. } => {
                then.requires_target()
                    || else_.as_ref().is_some_and(|e| e.requires_target())
            }
            Effect::Process { then, .. } => then.requires_target(),
            Effect::CollectEvidence { amount, then } => {
                value_has_target(amount) || then.requires_target()
            }
            Effect::CollectEvidenceX { then } => then.requires_target(),
            Effect::Forage { then } => then.requires_target(),
            Effect::Endure { target, n } => sel_has_target(target) || value_has_target(n),
            // Earthbend targets a land you control; blight chooses at resolution.
            Effect::Earthbend { .. } => true,
            Effect::Blight { .. } => false,
            Effect::Airbend { what } => sel_has_target(what),
            Effect::IfRevealFromHand { then, else_, .. } => {
                then.requires_target() || else_.requires_target()
            }
            Effect::DealDamage { to, amount }
            | Effect::EachControlledCreatureDealsDamage { to, amount } => {
                sel_has_target(to) || value_has_target(amount)
            }
            Effect::DealDamageExcessToController { to, amount } => {
                sel_has_target(to) || value_has_target(amount)
            }
            Effect::RadianceDamage { subject, amount }
            | Effect::SameNameDamage { subject, amount } => {
                sel_has_target(subject) || value_has_target(amount)
            }
            // Divided damage always targets (one or more chosen targets).
            Effect::DealDamageDivided { .. } => true,
            Effect::DealDamageDividedEvenly { .. } => true,
            Effect::CreateTokenBlocking { .. } => true,
            Effect::SupportCounters { .. } => true,
            Effect::DistributeCounters { .. } => true,
            Effect::ApplyToTargets { .. } => true,
            Effect::DeliverUntoEvil { .. } => true,
            Effect::FinaleOfPromise => true,
            Effect::Fight { attacker, defender } => {
                sel_has_target(attacker) || sel_has_target(defender)
            }
            Effect::DealDamageEqualToPower { source, target } => {
                sel_has_target(source) || sel_has_target(target)
            }
            Effect::DealDamageEqualToPowerToEach { source, targets, .. } => {
                sel_has_target(source) || sel_has_target(targets)
            }
            Effect::ExchangeControl { a, b } => sel_has_target(a) || sel_has_target(b),
            Effect::RedirectNextDamage { target, to, .. } => {
                sel_has_target(target) || sel_has_target(to)
            }
            Effect::ExchangeControlChoosing { with, .. } => sel_has_target(with),
            Effect::GainLife { who, amount } | Effect::LoseLife { who, amount } => {
                sel_has_target(who) || value_has_target(amount)
            }
            Effect::LoseHalfLife { who, .. }
            | Effect::MillHalf { who, .. }
            | Effect::MillThenDrawPerType { who, .. }
            | Effect::DiscardHalf { who, .. }
            | Effect::DoubleLife { who }
            | Effect::SacrificeHalf { who, .. } => sel_has_target(who),
            Effect::ShuffleSelfIntoLibrary => false,
            Effect::SetLifeTotal { who, amount } => {
                sel_has_target(who) || value_has_target(amount)
            }
            Effect::Learn { who } => player_has_target(who),
            Effect::ExchangeLifeTotals { a, b } => sel_has_target(a) || sel_has_target(b),
            Effect::Drain { from, to, amount } => {
                sel_has_target(from) || sel_has_target(to) || value_has_target(amount)
            }
            Effect::DiscardHandDrawThatMany { who } => sel_has_target(who),
            Effect::Draw { who, amount }
            | Effect::Mill { who, amount }
            | Effect::MillUntilLands { who, lands: amount }
            | Effect::ExileTopOfLibrary { who, amount, .. }
            | Effect::ExileTopMintPerChosenColor { who, amount, .. } => {
                sel_has_target(who) || value_has_target(amount)
            }
            Effect::MillTwoRepeatSharedColor { who } => sel_has_target(who),
            Effect::MillThenToHand { amount, .. } => value_has_target(amount),
            Effect::MillThenToHandN { amount, take, .. } => {
                value_has_target(amount) || value_has_target(take)
            }
            Effect::Discard { who, amount, .. } => sel_has_target(who) || value_has_target(amount),
            Effect::ExileFromHand { who, amount } => sel_has_target(who) || value_has_target(amount),
            Effect::CastUpToNFromOpponentsExile { count } => value_has_target(count),
            Effect::DiscardAnyNumber { who } => sel_has_target(who),
            Effect::SetNoMaxHandSize { who }
            | Effect::PutCardFromHandOnTopOfLibrary { who } => sel_has_target(who),
            Effect::SetMaxHandSize { who, size } => sel_has_target(who) || value_has_target(size),
            Effect::Scry { who, amount }
            | Effect::Surveil { who, amount }
            | Effect::LookAtTop { who, amount }
            | Effect::RearrangeTop { who, amount } => {
                player_has_target(who) || value_has_target(amount)
            }
            Effect::LookPickToHand { who, count, .. } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::RevealTopTakeOnePerType { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::RevealTopTakeMatchingToHand { who, count, .. }
            | Effect::RevealTopTakeMatchingRestToGraveyard { who, count, .. } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::ExileLibraryExceptBottom { who, keep } => {
                player_has_target(who) || value_has_target(keep)
            }
            Effect::Explore { who } => sel_has_target(who),
            Effect::Goad { what } => sel_has_target(what),
            Effect::Suspect { what } | Effect::ClearSuspected { what } => sel_has_target(what),
            Effect::Detain { what } => sel_has_target(what),
            Effect::Fateseal { who, amount } => {
                player_has_target(who) || value_has_target(amount)
            }
            Effect::DigToHandLoseLife { count, life_per_card } => {
                value_has_target(count) || value_has_target(life_per_card)
            }
            Effect::Discover { n, .. } => value_has_target(n),
            Effect::Monstrosity { n } => value_has_target(n),
            Effect::Move { what, to } => sel_has_target(what) || zonedest_has_target(to),
            Effect::MoveChosen { from, to, .. } => sel_has_target(from) || zonedest_has_target(to),
            Effect::Search { who, to, .. }
            | Effect::SearchLibraryOrGraveyard { who, to, .. } => {
                player_has_target(who) || zonedest_has_target(to)
            }
            Effect::SearchUpToN { who, to, .. } => {
                player_has_target(who) || zonedest_has_target(to)
            }
            Effect::SearchPickedBy { who, picker, to, .. } => {
                player_has_target(who) || player_has_target(picker) || zonedest_has_target(to)
            }
            Effect::Seek { who, to, count, .. } => {
                player_has_target(who) || zonedest_has_target(to) || value_has_target(count)
            }
            Effect::ReturnRandomFromGraveyard { who, count, .. } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::ShuffleGraveyardIntoLibrary { who }
            | Effect::ShuffleHandAndGraveyardIntoLibrary { who }
            | Effect::ShuffleFilteredGraveyardIntoLibraryGainLife { who, .. } => {
                player_has_target(who)
            }
            Effect::ExchangeHandAndGraveyard { who } => player_has_target(who),
            Effect::ShuffleLibrary { who } => player_has_target(who),
            Effect::SearchSplitOpponentChooses { opponent, .. } => sel_has_target(opponent),
            Effect::RedirectSpellTargetToSelf { what } => sel_has_target(what),
            Effect::RedirectYourDamageToChosen { what } => sel_has_target(what),
            Effect::AddManaKeptThisTurn { who, .. }
            | Effect::AddManaKeptThisTurnCount { who, .. } => player_has_target(who),
            Effect::AddManaEqualToPermanentCost { .. } => false,
            Effect::AddMana { who, pool } => {
                player_has_target(who) || match pool {
                    ManaPayload::Colorless(v)
                    | ManaPayload::AnyOneColor(v)
                    | ManaPayload::AnyColors(v) => value_has_target(v),
                    ManaPayload::OfColor(_, v) | ManaPayload::OfColors(_, v) => value_has_target(v),
                    ManaPayload::Restricted(inner, _)
                    | ManaPayload::RestrictedToChosenType(inner)
                    | ManaPayload::RestrictedToChosenTypePlain(inner) => match inner.as_ref() {
                        ManaPayload::Colorless(v)
                        | ManaPayload::AnyOneColor(v)
                        | ManaPayload::AnyColors(v)
                        | ManaPayload::OfColor(_, v)
                        | ManaPayload::OfColors(_, v) => value_has_target(v),
                        _ => false,
                    },
                    ManaPayload::Colors(_)
                    | ManaPayload::DevotionOfChosenColor
                    | ManaPayload::ChosenColorOfSource
                    | ManaPayload::ImprintedCardColor
                    | ManaPayload::AnyColorOpponentCouldProduce
                    | ManaPayload::AnyColorYouCouldProduce
                    | ManaPayload::AnyColorAmongLegendaries => false,
                }
            }
            Effect::Destroy { what }
            | Effect::DestroyAndRemember { what }
            | Effect::DestroyNoRegen { what }
            | Effect::Regenerate { what }
            | Effect::ExileIfWouldDieThisTurn { what }
            | Effect::GrantFlashbackThisTurn { what }
            | Effect::GrantHarmonizeThisTurn { what }
            | Effect::GrantMiracle { what, .. }
            | Effect::Exile { what }
            | Effect::ExileWithSource { what }
            | Effect::RemoveAllCountersDiscountNextSpell { what }
            | Effect::ExileSameNameAsTarget { what }
            | Effect::ExileTaggedWithSource { what }
            | Effect::ExileUntilSourceLeaves { what, .. }
            | Effect::ExileUntilOpponentMonarch { what }
            | Effect::ExileReturnNextEndStep { what }
            | Effect::FlipUntilLossThenTokenCopies { what }
            | Effect::ExileReturnToOwnerNextEndStep { what }
            | Effect::PhaseOut { what, .. }
            | Effect::GrantSuspend { what, .. }
            | Effect::ModularCounters { what }
            | Effect::Tap { what }
            | Effect::TapAndUntapLock { what }
            | Effect::RemoveFromCombat { what }
            | Effect::Untap { what, .. }
            | Effect::Provoke { what }
            | Effect::MustBlockSource { what }
            | Effect::CounterSpell { what }
            | Effect::CounterSpellExileSameNamed { what }
            | Effect::CounterSpellDrawIfUnderpaid { what }
            | Effect::CounterSpellToZone { what, .. }
            | Effect::CounterSpellExileNameLock { what }
            | Effect::CounterAbility { what }
            | Effect::CounterSpellOrAbility { what }
            | Effect::CounterUnlessPaid { what, .. }
            | Effect::CounterUnless { what, .. }
            | Effect::MakeSpellUncounterable { what } => sel_has_target(what),
            Effect::UnlessPlayerPays { then, .. } => then.requires_target(),
            Effect::PumpPT { what, power, toughness, .. } => {
                sel_has_target(what) || value_has_target(power) || value_has_target(toughness)
            }
            Effect::DoublePower { what, times, .. } => {
                sel_has_target(what) || value_has_target(times)
            }
            Effect::SetBasePT { what, power, toughness, .. } => {
                sel_has_target(what) || value_has_target(power) || value_has_target(toughness)
            }
            Effect::SwitchPT { what, .. } => sel_has_target(what),
            Effect::BecomeCreature { what, power, toughness, .. } => {
                sel_has_target(what) || value_has_target(power) || value_has_target(toughness)
            }
            Effect::GrantKeyword { what, .. } => sel_has_target(what),
            Effect::GrantKeywords { what, .. } => sel_has_target(what),
            Effect::AnimateAsCreature { what, .. } => sel_has_target(what),
            Effect::SetBasePower { what, power, .. } => {
                sel_has_target(what) || value_has_target(power)
            }
            Effect::LoseKeywordThisTurn { what, .. } => sel_has_target(what),
            Effect::SkipNextUntap { what } => sel_has_target(what),
            Effect::SkipPlayerUntapStep { player } => player_has_target(player),
            Effect::LandsDontUntapNextUntapStep { who } => sel_has_target(who),
            Effect::SacrificeAllMatching { who, .. } => sel_has_target(who),
            Effect::LivingDeath => false,
            Effect::SacrificeOthersThenReanimate => false,
            Effect::EscalatingThisTurn { modes } => modes.iter().any(|e| e.requires_target()),
            Effect::EachPlayerMayPutPermanentFromHand { .. } => false,
            Effect::BecomeChosenColor { what, .. }
            | Effect::BecomeColor { what, .. }
            | Effect::BecomeCreatureType { what, .. }
            | Effect::AddCreatureTypes { what, .. }
            | Effect::ReplaceColorWord { what, .. }
            | Effect::ReplaceBasicLandType { what, .. }
            | Effect::GrantProtectionFromChosenColor { what, .. } => sel_has_target(what),
            Effect::ChooseColorForSelf => false,
            Effect::Populate { .. } => false,
            Effect::LoseAllAbilities { what, .. } => sel_has_target(what),
            Effect::AddCounter { what, amount, .. }
            | Effect::RemoveCounter { what, amount, .. }
            | Effect::AddKeywordCounter { what, amount, .. }
            | Effect::RemoveKeywordCounter { what, amount, .. } => {
                sel_has_target(what) || value_has_target(amount)
            }
            Effect::AddRandomMissingCounter { what, .. } => sel_has_target(what),
            // Untargeted fan-out over creatures you control (Okinec Ahau).
            Effect::AddCountersForPowerOverBase { .. } => false,
            Effect::MoveAllCounters { from, to } => sel_has_target(from) || sel_has_target(to),
            Effect::MoveCounter { from, to, amount, .. } => {
                sel_has_target(from) || sel_has_target(to) || value_has_target(amount)
            }
            Effect::RemoveAllCounters { what } | Effect::RemoveAnyCounter { what } => sel_has_target(what),
            Effect::RemoveCountersUpTo { what, amount } => sel_has_target(what) || value_has_target(amount),
            Effect::SetLoyalty { what, value } => sel_has_target(what) || value_has_target(value),
            Effect::GrantLoyaltyTwiceThisTurn { what }
            | Effect::BecomeTreasure { what }
            | Effect::AddCounterOfPresentKind { what } => sel_has_target(what),
            Effect::Proliferate => false,
            Effect::BlockersPoisonedThisTurn { .. } => false,
            Effect::AuraSwapFromHand => false,
            // Targets slot 0 (a creature) but reads it straight off ctx.
            Effect::PreventNextDamageByTargetMintMites => true,
            Effect::GainControl { what, .. }
            | Effect::GainControlWhileSourceRemains { what }
            | Effect::WeldArtifacts { what } => sel_has_target(what),
            Effect::CreateToken { who, count, .. }
            | Effect::CreateTokenAttacking { who, count, .. }
            | Effect::Amass { who, count, .. } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::Incubate { who, amount } => {
                player_has_target(who) || value_has_target(amount)
            }
            Effect::BecomeBasicLand { what, .. }
            | Effect::GainAllBasicLandTypes { what, .. }
            | Effect::ResetCreature { what, .. } => sel_has_target(what),
            Effect::BecomeCopyOf { what, source, .. }
            | Effect::BecomeCopyOfFor { what, source, .. } => {
                sel_has_target(what) || sel_has_target(source)
            }
            Effect::Attach { what, to } => sel_has_target(what) || sel_has_target(to),
            Effect::CopySpell { what, count }
            | Effect::CopySpellWithRiders { what, count, .. }
            | Effect::CopySpellMayChooseTargets { what, count } => {
                sel_has_target(what) || value_has_target(count)
            }
            Effect::ChooseNewTargetsForSpell { what } => sel_has_target(what),
            Effect::CopySpellUnlessPaid { what, count, .. } => {
                sel_has_target(what) || value_has_target(count)
            }
            Effect::GrantMayPlay { what, .. }
            | Effect::StampMayPlaySurcharge { what, .. } => sel_has_target(what),
            Effect::GrantCastBackFromGraveyard { what } => sel_has_target(what),
            Effect::GainActivatedAbility { what, .. } => sel_has_target(what),
            Effect::AddCardTypeIndefinitely { what, .. } => sel_has_target(what),
            Effect::CastWithoutPayingImmediate { what, .. } => sel_has_target(what),
            Effect::RegisterParadigm | Effect::CastFreeParadigmCopy => false,
            Effect::Cascade { .. } => false,
            Effect::Ripple { .. } => false,
            Effect::Sacrifice { who, count, .. } => sel_has_target(who) || value_has_target(count),
            Effect::PlayerExilesPermanents { count, .. }
            | Effect::PlayerReturnsPermanentsToHand { count, .. } => value_has_target(count),
            Effect::SacrificeGreatestMV { who, count, .. } => {
                sel_has_target(who) || value_has_target(count)
            }
            Effect::Punisher { chooser, options, otherwise } => {
                sel_has_target(chooser)
                    || options.iter().any(|e| e.requires_target())
                    || otherwise.requires_target()
            }
            Effect::VillainousChoice { who, option_a, option_b } => {
                sel_has_target(who)
                    || option_a.requires_target()
                    || option_b.requires_target()
            }
            Effect::AddPoison { who, amount } => sel_has_target(who) || value_has_target(amount),
            Effect::AddRadCounters { who, amount } => {
                sel_has_target(who) || value_has_target(amount)
            }
            Effect::RevealTopAndDrawIf { who, .. }
            | Effect::RevealTopCard { who }
            | Effect::RevealTopLandToBattlefieldElseHand { who }
            | Effect::LookTopLandToHandElseBin { who }
            | Effect::RevealTopPutPermanentMvElseHand { who, .. }
            | Effect::RevealTopNPutMatchingToBattlefield { who, .. }
            | Effect::RevealTopPutPermanentOntoBattlefield { who } => {
                player_has_target(who)
            }
            Effect::RevealTopThenIf { who, then, .. } => {
                player_has_target(who) || then.requires_target()
            }
            Effect::RevealTopOpponentChoosesToHand { .. }
            | Effect::ReturnFromExileWithCounter { .. } => false,
            Effect::BecomeMonarch { who } | Effect::Ascend { who } => player_has_target(who),
            Effect::BecomeDay | Effect::BecomeNight | Effect::EndTheTurn => false,
            Effect::PreventAllDamageFromChosenSourceThisTurn { .. } => false,
            Effect::ExileSelfReturnTransformed => false,
            Effect::PutOnLibraryFromHand { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::DelayUntil { body, .. } => body.requires_target(),
            // Needs a creature to watch for death (the watched target).
            Effect::WhenTargetDiesThisTurn { .. } => true,
            // Needs a creature to watch for damage (Paladin's Forecast).
            Effect::GainLifeWhenTargetDealsDamageThisTurn { .. } => true,
            // Registers a turn-scoped delayed trigger; no cast-time target.
            Effect::CreaturesYouControlEnteringThisTurn { .. } => false,
            Effect::EachPlayerReanimateCreatureMaxMv { .. } => false,
            Effect::CreaturesYouControlDyingThisTurn { .. } => false,
            Effect::WheneverCreatureDiesThisTurn { .. } => false,
            Effect::CreaturesYouControlDealingCombatDamageThisTurn { .. } => false,
            Effect::WheneverYouGainLifeThisTurn { .. } => false,
            Effect::WheneverCardEntersOpponentGraveyardThisTurn { .. } => false,
            Effect::MayExileSelfReturnNextUpkeepHaste => false,
            Effect::PayOrLoseGame { .. } => false,
            Effect::SacrificeAndRemember { who, .. } => player_has_target(who),
            Effect::SacrificeAnyNumber { per_each, .. } => per_each.requires_target(),
            Effect::PayLifeLookTake { .. } => false,
            Effect::PayLifeDraw { .. } => false,
            Effect::RevealUntilLandDamage { to, .. } => sel_has_target(to),
            Effect::RevealUntilNonlandDamage { to } => sel_has_target(to),
            // Reveals from your own library — no target slot.
            Effect::RevealUntilLandsToBattlefield { .. } => false,
            Effect::ClashWithOpponent { .. } => false,
            Effect::OnAttackedUntilYourNextTurn { .. } => false,
            Effect::ExileAnyNumberFromGraveyards { .. } => false,
            Effect::MayExileFromYourGraveyard { then, .. } => then.requires_target(),
            Effect::ExileAllGraveyards { .. } => false,
            Effect::LivingEnd => false,
            Effect::ExilePlayerGraveyard { who } => player_has_target(who),
            Effect::AddFirstSpellTax { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::GrantSorceriesAsFlash { who } => player_has_target(who),
            Effect::GrantExtraLandPlay { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::RevealUntilFind { who, to, cap, .. } => {
                player_has_target(who)
                    || zonedest_has_target(to)
                    || value_has_target(cap)
            }
            Effect::DiscardChosen { from, count, .. }
            | Effect::BottomChosenFromHandAndDraw { from, count, .. }
            | Effect::ExileChosenUntilSourceLeaves { from, count, .. }
            | Effect::ExileChosenFromHand { from, count, .. } => {
                sel_has_target(from) || value_has_target(count)
            }
            Effect::NameCreatureType { what } => sel_has_target(what),
            Effect::NameCard { what } => sel_has_target(what),
            Effect::LockTargetNameUntilYourNextTurn { what } => sel_has_target(what),
            Effect::NameOpponentCastLock => false,
            Effect::WinGame { who } | Effect::LoseGame { who } => player_has_target(who),
            Effect::SkipTurns { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::SkipNextCombatPhase { who } => player_has_target(who),
            Effect::TakeExtraTurn { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::AdditionalCombatPhase { count }
            | Effect::AdditionalCombatPhaseAfterMain { count }
            | Effect::AdditionalEndStep { count }
            | Effect::AdditionalUpkeepStep { count } => value_has_target(count),
            // Registers a delayed trigger; its body targets at fire time, not cast.
            Effect::AtEachCombatThisTurn { .. } => false,
            Effect::UnlockRoomDoor { what } => sel_has_target(what),
            Effect::CreateEmblem { who, .. } => player_has_target(who),
            Effect::CreateTokenCopyOf { who, count, source, .. }
            | Effect::CreateTokenCopiesHasteSac { who, count, source } => {
                player_has_target(who) || value_has_target(count) || sel_has_target(source)
            }
            Effect::GrantTriggeredAbility { what, .. } => sel_has_target(what),
            Effect::PreventAllCombatDamageThisTurn => false,
            Effect::PreventCombatDamageExceptDealtBy { .. } => false,
            Effect::PreventAllCombatDamageToPlayerThisTurn { .. } => false,
            Effect::SacrificeSourceUnlessPayManaValue | Effect::SacrificeSourceUnlessPay { .. } => false,
            Effect::PreventAllCombatDamageInvolving { target } => sel_has_target(target),
            Effect::PreventCombatDamageToTargetThisTurn { target } => sel_has_target(target),
            Effect::PreventCombatDamageByTargetThisTurn { target } => sel_has_target(target),
            Effect::CantBlockSourceThisTurn { target } => sel_has_target(target),
            Effect::PreventNextDamage { target, amount }
            | Effect::PreventNextDamageAndGainLife { target, amount } => {
                sel_has_target(target) || value_has_target(amount)
            }
            Effect::PreventAllDamageThisTurn { target } => sel_has_target(target),
            Effect::ReplaceNextDamageWithDestroy { target } => sel_has_target(target),
            Effect::DamageCantBePreventedThisTurn => false,
            Effect::PreventSearchesThisTurn => false,
            Effect::PlayerProtectionUntilNextTurn { .. } => false,
            Effect::WhenLastCreatedTokenLeaves { .. } => false,
            Effect::DiminishCreaturesExceptChosenType { power, toughness } => {
                value_has_target(power) || value_has_target(toughness)
            }
            Effect::LifeGainLockThisTurn { who } | Effect::LifeLockThisTurn { who } => sel_has_target(who),
            Effect::LifeGainLockGame { who } => sel_has_target(who),
            Effect::GrantSpellsUncounterableThisTurn { who }
            | Effect::GrantCreatureSpellsUncounterableThisTurn { who } => sel_has_target(who),
            Effect::GrantHexproofFromColorThisTurn { who, .. } => sel_has_target(who),
            Effect::GainHexproofUntilYourNextTurn { who } => player_has_target(who),
            Effect::CantCastNoncreatureThisTurn { who } => sel_has_target(who),
            Effect::ExileTopAndGrantMayPlay { .. } => false,
            Effect::ExileTopLandTokenElseMayPlay { .. } => false,
            Effect::AddEnergy(amount) => value_has_target(amount),
            Effect::AddExperience(amount) => value_has_target(amount),
            Effect::PayEnergy { then, .. } | Effect::PayEnergyValue { then, .. } | Effect::PayAnyEnergy { then } => then.requires_target(),
            Effect::PayAnyEnergyDealDamage { to } => sel_has_target(to),
            Effect::TimeTravel { who } => player_has_target(who),
            Effect::PayEnergyOrElse { otherwise, .. }
            | Effect::PayEnergyOrElseValue { otherwise, .. } => otherwise.requires_target(),
            Effect::PayManaOrElse { otherwise, .. } => otherwise.requires_target(),
            Effect::ExileTopMayPayEnergyToCast { .. } => false,
            Effect::DoubleCountersOnEach { what, .. } => sel_has_target(what),
            Effect::DoubleAllCountersOn { what } => sel_has_target(what),
            Effect::SacrificePermanent { what } => sel_has_target(what),
            Effect::ExileLastCreatedTokensAtNextEndStep
            | Effect::SacrificeLastCreatedTokensAtNextEndStep => false,
            Effect::EchoPayOrSacrifice { .. } => false,
            Effect::CumulativeUpkeepPayOrSacrifice { .. } => false,
            Effect::Balance => false,
            Effect::GenesisWave => false,
            Effect::ShuffleHandsDrawSame { who } => player_has_target(who),
        }
    }

    /// Extract the target's filter if this effect's top-level "what"/"to" is
    /// `Selector::Target(0)`. Used by UI/bot for target selection.
    pub fn primary_target_filter(&self) -> Option<&SelectionRequirement> {
        fn sel_filter(s: &Selector) -> Option<&SelectionRequirement> {
            match s {
                Selector::EachMatching { filter, .. } => Some(filter),
                Selector::EachPermanent(f) => Some(f),
                Selector::CardsInZone { filter, .. } => Some(filter),
                Selector::TargetFiltered { filter, .. } => Some(filter),
                Selector::Take { inner, .. }
                | Selector::TakeRandom { inner, .. } => sel_filter(inner),
                Selector::TakeWithSumCap { inner, .. } => sel_filter(inner),
                Selector::RadianceGroup { subject } => sel_filter(subject),
                Selector::CreaturesInCombatWith(subject) => sel_filter(subject),
                _ => None,
            }
        }
        match self {
            // Prefer the damage target's own filter; fall back to a filter
            // hidden in the damage amount (Rabid Bite: `PowerOf(slot 0)`).
            Effect::DealDamage { to, amount }
            | Effect::EachControlledCreatureDealsDamage { to, amount } => {
                sel_filter(to).or_else(|| match amount {
                    Value::CountOf(s) | Value::PowerOf(s) | Value::ToughnessOf(s) => sel_filter(s),
                    _ => None,
                })
            }
            Effect::RadianceDamage { subject, .. }
            | Effect::SameNameDamage { subject, .. } => sel_filter(subject),
            Effect::CreateTokenBlocking { filter, .. }
            | Effect::DealDamageDivided { filter, .. }
            | Effect::DealDamageDividedEvenly { filter, .. }
            | Effect::DistributeCounters { filter, .. }
            | Effect::DestroyTargetsPolymorph { filter }
            | Effect::ApplyToTargets { filter, .. }
            | Effect::DeliverUntoEvil { filter, .. }
            | Effect::DestroyTargets { filter } => Some(filter),
            Effect::PayAnyEnergyDealDamage { to } => sel_filter(to),
            // Fight surfaces the *defender's* filter (the opp creature
            // we want to fight). The attacker is usually the friendly
            // already-on-bf source/target.
            Effect::Fight { defender, .. } => sel_filter(defender),
            Effect::DealDamageEqualToPower { target, .. } => sel_filter(target),
            // Land hosing targets the land slot (Tide Shaper's kicked mode).
            Effect::BecomeBasicLand { what, .. } => sel_filter(what),
            // The chosen creature (`source`) is the targeted object; the
            // per-creature/opponent recipients are not targeted.
            Effect::DealDamageEqualToPowerToEach { source, .. } => sel_filter(source),
            // The targeted side may be `b` when `a` is the source itself
            // (Volatile Stormdrake exchanges `This` with a targeted creature).
            Effect::ExchangeControl { a, b } => sel_filter(a).or_else(|| sel_filter(b)),
            Effect::RedirectNextDamage { target, to, .. } => {
                sel_filter(target).or_else(|| sel_filter(to))
            }
            Effect::ExchangeControlChoosing { with, .. } => sel_filter(with),
            Effect::GainLife { who, .. } | Effect::LoseLife { who, .. } => sel_filter(who),
            Effect::LoseHalfLife { who, .. }
            | Effect::MillHalf { who, .. }
            | Effect::MillThenDrawPerType { who, .. }
            | Effect::DiscardHalf { who, .. }
            | Effect::DoubleLife { who }
            | Effect::SacrificeHalf { who, .. } => sel_filter(who),
            Effect::SetLifeTotal { who, .. } => sel_filter(who),
            Effect::Destroy { what }
            | Effect::DestroyAndRemember { what }
            | Effect::DestroyNoRegen { what }
            | Effect::Regenerate { what }
            | Effect::ExileIfWouldDieThisTurn { what }
            | Effect::GrantFlashbackThisTurn { what }
            | Effect::GrantHarmonizeThisTurn { what }
            | Effect::GrantMiracle { what, .. }
            | Effect::Exile { what }
            | Effect::ExileWithSource { what }
            | Effect::RemoveAllCountersDiscountNextSpell { what }
            | Effect::ExileSameNameAsTarget { what }
            | Effect::ExileTaggedWithSource { what }
            | Effect::ExileUntilSourceLeaves { what, .. }
            | Effect::ExileUntilOpponentMonarch { what }
            | Effect::ExileReturnNextEndStep { what }
            | Effect::FlipUntilLossThenTokenCopies { what }
            | Effect::ExileReturnToOwnerNextEndStep { what }
            | Effect::Provoke { what }
            | Effect::MustBlockSource { what }
            | Effect::Suspect { what }
            | Effect::ClearSuspected { what }
            | Effect::Detain { what }
            | Effect::CounterSpell { what }
            | Effect::CounterSpellExileSameNamed { what }
            | Effect::CounterSpellDrawIfUnderpaid { what }
            | Effect::CounterSpellToZone { what, .. }
            | Effect::CounterSpellExileNameLock { what }
            | Effect::CounterAbility { what }
            | Effect::CounterSpellOrAbility { what }
            | Effect::CounterUnlessPaid { what, .. }
            | Effect::CounterUnless { what, .. }
            | Effect::MakeSpellUncounterable { what }
            | Effect::CastWithoutPayingImmediate { what, .. }
            | Effect::CopySpell { what, .. }
            | Effect::CopySpellWithRiders { what, .. }
            | Effect::CopySpellMayChooseTargets { what, .. }
            | Effect::GainControl { what, .. }
            | Effect::GainControlWhileSourceRemains { what }
            | Effect::WeldArtifacts { what } => sel_filter(what),
            // The target may be the moved object (`what`: Kor Outfitter's
            // "target Equipment") or the host (`to`: Maul's "attach this to
            // target creature"). Prefer whichever sub-selector carries slot 0.
            // Chef's Kiss targets a spell on the stack.
            Effect::ChefsKiss => {
                const F: SelectionRequirement = SelectionRequirement::IsSpellOnStack;
                Some(&F)
            }
            Effect::Attach { what, to } => sel_filter(what).or_else(|| sel_filter(to)),
            // "Tap all lands target player controls" surfaces the implicit
            // Player filter (Mistbind Clique); plain selectors keep theirs.
            Effect::PhaseOut { what, .. }
            | Effect::GrantSuspend { what, .. }
            | Effect::ModularCounters { what }
            | Effect::Tap { what }
            | Effect::SetSaddled { what }
            | Effect::TapAndUntapLock { what }
            | Effect::Untap { what, .. } => {
                sel_filter(what).or_else(|| implicit_player_if_controlled_by_target(what))
            }
            Effect::UnlessPlayerPays { then, .. } => then.primary_target_filter(),
            Effect::AddCounter { what, .. }
            | Effect::RemoveCounter { what, .. }
            | Effect::RemoveAllCounters { what }
            | Effect::RemoveAnyCounter { what }
            | Effect::RemoveCountersUpTo { what, .. }
            | Effect::SetLoyalty { what, .. }
            | Effect::GrantLoyaltyTwiceThisTurn { what }
            | Effect::BecomeTreasure { what }
            | Effect::AddCounterOfPresentKind { what }
            | Effect::AddKeywordCounter { what, .. }
            | Effect::RemoveKeywordCounter { what, .. }
            | Effect::AddRandomMissingCounter { what, .. } => sel_filter(what),
            // CreateTokenCopyOf — the `source` is the targeted permanent to
            // copy (Esika's Chariot "copy target token you control").
            Effect::CreateTokenCopyOf { source, .. }
            | Effect::CreateTokenCopiesHasteSac { source, .. } => sel_filter(source),
            // CreateTokenAttachedTo — the `target` is the creature the minted
            // Aura/Role token attaches to (Splashy Spellcaster's Role).
            Effect::CreateTokenAttachedTo { target, .. }
            | Effect::CreateTokenAttachedToEach { target, .. } => sel_filter(target),
            Effect::PumpPT { what, .. }
            | Effect::SetBasePT { what, .. }
            | Effect::SwitchPT { what, .. }
            | Effect::DoublePower { what, .. } => {
                sel_filter(what).or_else(|| implicit_creature_if_bare_target(what))
            }
            Effect::BecomeCreature { what, .. } => sel_filter(what),
            Effect::AnimateAsCreature { what, .. } => sel_filter(what),
            Effect::SetBasePower { what, .. } => sel_filter(what),
            Effect::GrantKeyword { what, .. }
            | Effect::GrantKeywords { what, .. }
            | Effect::ReplaceColorWord { what, .. }
            | Effect::ReplaceBasicLandType { what, .. }
            | Effect::GrantProtectionFromChosenColor { what, .. } => sel_filter(what),
            Effect::Move { what, .. } => sel_filter(what),
            // Player-targeting effects: surface the filter so the bot's
            // auto-target heuristic can find the opp / caster without a
            // manual Target. The filter is typically `Player` (Mind Rot,
            // Sign in Blood) but can be narrower (Howling Mine-style "you").
            Effect::Discard { who, .. }
            | Effect::DiscardAnyNumber { who }
            | Effect::SetNoMaxHandSize { who }
            | Effect::SetMaxHandSize { who, .. }
            | Effect::Draw { who, .. }
            | Effect::Mill { who, .. }
            | Effect::MillUntilLands { who, .. }
            | Effect::MillTwoRepeatSharedColor { who }
            | Effect::ExileTopOfLibrary { who, .. } => sel_filter(who),
            Effect::ExileTopMintPerChosenColor { who, .. } => {
                sel_filter(who).or_else(|| implicit_player_if_bare_player_target(who))
            }
            Effect::Drain { to, .. } => sel_filter(to),
            Effect::AddPoison { who, .. } => sel_filter(who),
            Effect::DiscardChosen { from, .. } => sel_filter(from),
            Effect::BottomChosenFromHandAndDraw { from, .. } => sel_filter(from),
            Effect::SearchSplitOpponentChooses { opponent, .. } => sel_filter(opponent),
            Effect::RedirectSpellTargetToSelf { what } => sel_filter(what),
            Effect::RedirectYourDamageToChosen { what } => sel_filter(what),
            Effect::ManaClash { opponent } => sel_filter(opponent),
            // Edict-class effects: "target player sacrifices a permanent."
            // The `who` selector usually carries a `target_filtered(Player)`
            // filter (Sudden Edict, Cruel Edict-style spells); bare
            // `Selector::Target(0)` falls through unchanged so existing
            // edicts that pre-date the filter primitive (Diabolic Edict,
            // Geth's Verdict) keep their explicit-target casting contract.
            Effect::Sacrifice { who, .. } | Effect::SacrificeGreatestMV { who, .. } => {
                sel_filter(who)
            }
            // Compound effects: walk into the children. Spells like Goryo's
            // Vengeance wrap a `Move` (target legendary creature) in a
            // `Seq` alongside a delayed exile trigger; the primary target
            // is still the Move's target.
            Effect::Seq(v) => v.iter().find_map(|e| e.primary_target_filter()),
            Effect::If { then, else_, .. } => then
                .primary_target_filter()
                .or_else(|| else_.primary_target_filter()),
            Effect::DelayUntil { body, .. } => body.primary_target_filter(),
            Effect::OptionalTargets { body, .. } => body.primary_target_filter(),
            // The copy *source* is the targeted slot ("becomes a copy of
            // target land").
            Effect::BecomeCopyOf { source, .. }
            | Effect::BecomeCopyOfFor { source, .. } => sel_filter(source),
            Effect::WhenTargetDiesThisTurn { filter, .. } => {
                filter.as_ref().or(Some(&SelectionRequirement::Creature))
            }
            Effect::GainLifeWhenTargetDealsDamageThisTurn { .. } => {
                Some(&SelectionRequirement::Creature)
            }
            // Modal cards: surface the first mode's filter as the
            // representative one (UI/bot still need *some* filter to
            // narrow target candidates). Mode-specific validation lives
            // in `target_filter_for_slot_in_mode`, which the cast paths
            // consult once the user/bot has picked a mode.
            Effect::ChooseMode(modes) => modes
                .iter()
                .find_map(|e| e.primary_target_filter()),
            Effect::ChooseN { modes, .. } => modes
                .iter()
                .find_map(|e| e.primary_target_filter()),
            Effect::Escalate { modes, .. } => modes
                .iter()
                .find_map(|e| e.primary_target_filter()),
            // MayDo wraps an inner effect — surface its filter so the
            // cast prompt narrows correctly when the inner effect needs
            // a target (e.g. "you may sacrifice [target permanent]").
            Effect::MayDo { body, .. } | Effect::MayPayX { body, .. } | Effect::CapTargetsAtX { body } => body.primary_target_filter(),
            Effect::MayPay { body, .. } | Effect::MayPayLife { body, .. } => body.primary_target_filter(),
            Effect::PayEnergy { then, .. } | Effect::PayEnergyValue { then, .. } | Effect::PayAnyEnergy { then } => then.primary_target_filter(),
            Effect::Process { then, .. } => then.primary_target_filter(),
            Effect::CollectEvidence { then, .. }
            | Effect::CollectEvidenceX { then }
            | Effect::Forage { then } => {
                then.primary_target_filter()
            }
            Effect::WithSacrificedPt { body, .. }
            | Effect::WithTappedPower { body, .. }
            | Effect::OnYourNextSpellCastThisTurn { body }
            | Effect::OnYourNextInstantSorceryThisTurn { body }
            | Effect::OnYourNextNamedSpellThisTurn { body }
            | Effect::Repeat { body, .. }
            | Effect::ForEach { body, .. } => body.primary_target_filter(),
            Effect::Endure { target, .. } => sel_filter(target),
            // Earthbend targets a land you control (CR 701.66a).
            Effect::Earthbend { .. } => Some(&EARTHBEND_TARGET),
            Effect::Airbend { what } => sel_filter(what),
            Effect::Goad { what }
            | Effect::Transform { what }
            | Effect::Flip { what }
            | Effect::LoseAllAbilities { what, .. }
            | Effect::LoseKeywordThisTurn { what, .. }
            | Effect::SkipNextUntap { what }
            | Effect::GrantTriggeredAbility { what, .. }
            | Effect::GainActivatedAbility { what, .. }
            | Effect::AddCardTypeIndefinitely { what, .. }
            | Effect::BecomeChosenColor { what, .. }
            | Effect::BecomeColor { what, .. }
            | Effect::BecomeCreatureType { what, .. }
            | Effect::AddCreatureTypes { what, .. }
            | Effect::GrantMayPlay { what, .. }
            | Effect::StampMayPlaySurcharge { what, .. }
            | Effect::DoubleCountersOnEach { what, .. }
            | Effect::DoubleAllCountersOn { what }
            | Effect::NameCreatureType { what }
            | Effect::NameCard { what }
            | Effect::LockTargetNameUntilYourNextTurn { what }
            | Effect::Explore { who: what } => sel_filter(what),
            Effect::MoveAllCounters { from, to } | Effect::MoveCounter { from, to, .. } => {
                sel_filter(from).or_else(|| sel_filter(to))
            }
            Effect::Tribute { otherwise, .. } => otherwise.primary_target_filter(),
            Effect::TemptingOffer { body } => body.primary_target_filter(),
            Effect::PlayersMayAccept { otherwise, .. } => otherwise.primary_target_filter(),
            Effect::Punisher { options, otherwise, .. } => options
                .iter()
                .find_map(|e| e.primary_target_filter())
                .or_else(|| otherwise.primary_target_filter()),
            Effect::NthResolutionThisTurn { branches } => {
                branches.iter().find_map(|e| e.primary_target_filter())
            }
            Effect::SacrificeAnyNumber { per_each, .. } => per_each.primary_target_filter(),
            Effect::IfRevealFromHand { then, else_, .. } => then
                .primary_target_filter()
                .or_else(|| else_.primary_target_filter()),
            // FlipCoin: surface the heads branch first (the active
            // outcome) — same pattern as If/IfRevealFromHand. Falls back
            // to the tails branch if heads has no target.
            Effect::FlipCoin { on_heads, on_tails, .. } => on_heads
                .primary_target_filter()
                .or_else(|| on_tails.primary_target_filter()),
            // RollDie: surface the first results arm's filter as the
            // representative one (mirrors ChooseMode's pattern). The
            // auto-target picker walks the result-table arm that fires
            // for the rolled face; we surface the first arm for the
            // cast prompt.
            Effect::FlipCoinsUntilLoseOrStop { tiers } => {
                tiers.iter().find_map(|(_, e)| e.primary_target_filter())
            }
            Effect::RollDie { results, .. } => results
                .iter()
                .find_map(|(_, _, e)| e.primary_target_filter()),
            _ => None,
        }
    }

    /// Heuristic: does this effect's primary target want to be a *friendly*
    /// permanent (one the caster controls) rather than an opponent's? Drives
    /// `auto_target_for_effect` so the random bot doesn't waste Vines of
    /// Vastwood on the opp's bear or Reckless Charge on its own.
    ///
    /// Returns true for unconditional self-buffs (positive `PumpPT`,
    /// `GrantKeyword`, `+1/+1` `AddCounter`). Returns false for hostile
    /// effects (Destroy, Exile, DealDamage, …) and ambiguous ones.
    /// Friendliness of the children that actually declare a target slot,
    /// falling back to every child when none of them target.
    fn friendliness_of_targeting_children(children: &[Effect]) -> bool {
        let mut targeting = children.iter().filter(|e| e.requires_target()).peekable();
        if targeting.peek().is_some() {
            targeting.any(|e| e.prefers_friendly_target())
        } else {
            children.iter().any(|e| e.prefers_friendly_target())
        }
    }

    pub fn prefers_friendly_target(&self) -> bool {
        match self {
            Effect::PumpPT { power, toughness, .. } => {
                // Pump is friendly when the bonus is non-negative; debuffs
                // (Tragic Slip, Last Gasp) want opponent targets.
                Self::value_is_non_negative(power) && Self::value_is_non_negative(toughness)
            }
            // SetBasePT to 0/N (Square Up) is hostile when the base
            // power drops below the printed body — used as a removal-
            // adjacent effect to neutralize attackers. The bot prefers
            // an opp creature unless the toughness bump is the bigger
            // tell.
            Effect::SetBasePT { .. } => false,
            // Animating your own land into a creature is a friendly self-buff.
            Effect::BecomeCreature { .. } => true,
            Effect::AnimateAsCreature { .. } => true,
            // "Base power becomes equal to …" is a self-pump (Belligerent Yearling).
            Effect::SetBasePower { .. } => true,
            // Doubling a life total is a gift — point Beacon of Immortality at
            // the caster, not the opponent.
            Effect::DoubleLife { .. } => true,
            // Copying "target token you control" is friendly (Esika's Chariot).
            Effect::CreateTokenCopyOf { .. } | Effect::CreateTokenCopiesHasteSac { .. } => true,
            Effect::GrantKeyword { keyword, .. } => Self::keyword_is_friendly(keyword),
            Effect::GrantKeywords { keywords, .. } => {
                keywords.iter().any(Self::keyword_is_friendly)
            }
            Effect::AddCounter { kind, .. } => matches!(kind, CounterType::PlusOnePlusOne),
            // Only the sub-effects that actually surface a target slot get a
            // say: a non-targeting friendly prelude (Ordeal of Purphoros's
            // "+1/+1 counter on it") must not aim the hostile payload ("3
            // damage to any target") at its own controller. With no targeting
            // child, fall back to the whole batch's flavor.
            Effect::Seq(v) => Self::friendliness_of_targeting_children(v),
            Effect::If { then, else_, .. } => Self::friendliness_of_targeting_children(
                std::slice::from_ref(then.as_ref())
                    .iter()
                    .chain(std::slice::from_ref(else_.as_ref()))
                    .cloned()
                    .collect::<Vec<_>>()
                    .as_slice(),
            ),
            Effect::DelayUntil { body, .. } | Effect::Repeat { body, .. } => {
                body.prefers_friendly_target()
            }
            Effect::ForEach { body, .. }
            | Effect::MayDo { body, .. }
            | Effect::CapTargetsAtX { body }
            | Effect::MayPayX { body, .. } => {
                body.prefers_friendly_target()
            }
            // "TARGET player draws a card" is a gift — aim slot 0 at the
            // caster (Shadrix Silverquill's draw mode is the mode you take
            // yourself in the canonical two-pick line). A non-targeted
            // Draw (who: You / EachPlayer) declares no preference, so
            // hostile siblings (Keranos's reveal-bolt branch) keep their
            // opponent-facing default.
            Effect::Draw { who, .. } => matches!(
                who,
                Selector::Player(crate::effect::PlayerRef::Target(_))
                    | Selector::Target(_)
                    | Selector::TargetFiltered { .. }
            ),
            // Modal: friendly when the first slot-owning mode (default
            // picks order — slot 0's owner) is friendly.
            Effect::ChooseN { picks, modes } => picks
                .iter()
                .filter_map(|&i| modes.get(i as usize))
                .find(|m| m.requires_target())
                .is_some_and(|m| m.prefers_friendly_target()),
            Effect::Process { then, .. } => then.prefers_friendly_target(),
            // Reanimate-style spells move target → caster's hand or battlefield.
            // Without this, `auto_target_for_effect` picks an opp's battlefield
            // creature first, and Disentomb / Raise Dead happily steal it.
            Effect::Move { to, .. } => matches!(
                to,
                ZoneDest::Hand(PlayerRef::You)
                    | ZoneDest::Battlefield { controller: PlayerRef::You, .. }
            ),
            _ => false,
        }
    }

    /// This effect's primary target is meant to be a card in *some*
    /// graveyard. Covers two cases:
    /// - Reanimate-class (Disentomb, Raise Dead, Reanimate, Goryo's
    ///   Vengeance) — `Move target → Hand(You)` / `Battlefield(You)`.
    /// - Graveyard hate (Ghost Vacuum's "exile target card from a
    ///   graveyard") — `Move target → Exile`.
    ///
    /// The auto-target heuristic walks graveyards (in friendly/hostile
    /// order) before the battlefield when this is set, so an `Any`-filtered
    /// Move-to-Exile picks a graveyard resident rather than a battlefield
    /// permanent that happens to be at the top of the scan.
    ///
    /// Battlefield Move-to-Exile is rare in the catalog (the canonical
    /// permanent-exile effect is `Effect::Exile`), so collapsing both
    /// graveyard-walk cases under one classifier is safe.
    pub fn prefers_graveyard_target(&self) -> bool {
        match self {
            Effect::Move { to, .. } => matches!(
                to,
                ZoneDest::Hand(PlayerRef::You)
                    | ZoneDest::Battlefield { controller: PlayerRef::You, .. }
                    | ZoneDest::Exile
            ),
            Effect::Seq(v) => v.iter().any(|e| e.prefers_graveyard_target()),
            Effect::If { then, else_, .. } => {
                then.prefers_graveyard_target() || else_.prefers_graveyard_target()
            }
            Effect::DelayUntil { body, .. }
            | Effect::Repeat { body, .. }
            | Effect::ForEach { body, .. }
            | Effect::MayDo { body, .. }
            | Effect::CapTargetsAtX { body }
            | Effect::MayPayX { body, .. }
            | Effect::MayPay { body, .. }
            | Effect::MayPayLife { body, .. } => body.prefers_graveyard_target(),
            Effect::Process { then, .. } => then.prefers_graveyard_target(),
            // Recasting a target card *from the graveyard* (Efreet Flamepainter,
            // The Dawning Archaic) wants the graveyard walked for the target.
            Effect::CastWithoutPayingImmediate { source_zone, .. } => {
                matches!(source_zone, crate::card::Zone::Graveyard)
            }
            // Granting flashback to a card always targets one in a graveyard
            // (Snapcaster Mage, Slickshot Lockpicker).
            Effect::GrantFlashbackThisTurn { .. }
            | Effect::GrantHarmonizeThisTurn { .. } => true,
            _ => false,
        }
    }

    /// The slot-0 target phrase for label text — "target creature",
    /// "any target", or a plain "target" when the slot-0 filter has no clean
    /// noun (complex `Or`/`Not`/stat gates) or there is no slot-0 target.
    /// Lets `effect_short_text` surface a target's restriction so a modal like
    /// Abrade reads "destroy target artifact" rather than "destroy target".
    fn target_phrase(&self) -> String {
        match self.target_filter_for_slot(0) {
            Some(crate::card::SelectionRequirement::Any) => "any target".to_string(),
            Some(req) => match req.target_noun() {
                Some(noun) => format!("target {noun}"),
                None => "target".to_string(),
            },
            None => "target".to_string(),
        }
    }

    /// Short human-readable summary of this effect's target shape, used
    /// in trigger prompts ("<source name> — exile target card from a
    /// graveyard"). Covers the common cases (Move-to-zone, Destroy,
    /// Exile, AddCounter, DealDamage, PumpPT); returns an empty string
    /// for effect shapes that aren't worth phrasing. Walks into Seq /
    /// If / MayDo / ForEach to find the first informative inner effect.
    pub fn effect_short_text(&self) -> String {
        match self {
            Effect::Move { to, .. } => {
                let t = self.target_phrase();
                match to {
                    ZoneDest::Exile => format!("exile {t}"),
                    ZoneDest::Hand(_) => format!("return {t} to its owner's hand"),
                    ZoneDest::Graveyard => format!("put {t} into its owner's graveyard"),
                    ZoneDest::Battlefield { .. } => format!("put {t} onto the battlefield"),
                    ZoneDest::Library { .. } => format!("put {t} into its owner's library"),
                }
            }
            Effect::Destroy { .. } | Effect::DestroyAndRemember { .. } => {
                format!("destroy {}", self.target_phrase())
            }
            Effect::DestroyLandOfEachBasicType => {
                "choose a land of each basic land type, then destroy those lands".into()
            }
            Effect::ExilePlayerGraveyard { who } => match who {
                crate::effect::PlayerRef::EachOpponent => "exile each opponent's graveyard".into(),
                crate::effect::PlayerRef::You => "exile your graveyard".into(),
                _ => "exile target player's graveyard".into(),
            },
            Effect::Spree { .. } => "spree (choose one or more additional costs)".into(),
            Effect::Tiered { .. } => "tiered (choose one additional cost)".into(),
            Effect::ChooseModesCast { min, max, .. } => {
                format!("choose {min}-{max} modes")
            }
            Effect::DestroyNoRegen { .. } => {
                format!("destroy {} (can't be regenerated)", self.target_phrase())
            }
            Effect::Exile { .. } => format!("exile {}", self.target_phrase()),
            Effect::ExileUntilSourceLeaves { .. } => {
                format!("exile {} until this leaves the battlefield", self.target_phrase())
            }
            Effect::ExileUntilOpponentMonarch { .. } => {
                format!("exile {} until an opponent becomes the monarch", self.target_phrase())
            }
            Effect::DealDamage { amount, .. } => {
                let t = self.target_phrase();
                match amount {
                    Value::Const(n) => format!("deal {n} damage to {t}"),
                    _ => format!("deal damage to {t}"),
                }
            }
            Effect::DealDamageExcessToController { amount, .. } => {
                let t = self.target_phrase();
                match amount {
                    Value::Const(n) => format!(
                        "deal {n} damage to {t}; excess is dealt to its controller"),
                    _ => format!("deal damage to {t}; excess is dealt to its controller"),
                }
            }
            Effect::DealDamageDivided { total, .. } => match total {
                Value::Const(n) => format!("deal {n} damage divided among targets"),
                _ => "deal damage divided among targets".into(),
            },
            Effect::DealDamageDividedEvenly { total, .. } => match total {
                Value::Const(n) => format!("deal {n} damage divided evenly among targets"),
                _ => "deal damage divided evenly among targets".into(),
            },
            Effect::DistributeCounters { total, counter, .. } => match total {
                Value::Const(n) => format!("distribute {n} {counter:?} counters among targets"),
                _ => "distribute counters among targets".into(),
            },
            Effect::ApplyToTargets { effect, .. } => {
                format!("{} (each of up to N targets)", effect.effect_short_text())
            }
            Effect::AddCounter { kind, amount, .. } => {
                let t = self.target_phrase();
                match amount {
                    Value::Const(n) => format!("put {n} {kind:?} counter(s) on {t}"),
                    _ => format!("put {kind:?} counter(s) on {t}"),
                }
            }
            Effect::PumpPT { power, toughness, .. } => {
                let t = self.target_phrase();
                match (power, toughness) {
                    (Value::Const(p), Value::Const(tn)) => {
                        format!("{t} gets {p:+}/{tn:+} until end of turn")
                    }
                    _ => format!("pump {t} until end of turn"),
                }
            }
            Effect::Tap { .. } | Effect::TapAndUntapLock { .. } => format!("tap {}", self.target_phrase()),
            Effect::PhaseOut { .. } => format!("phase out {}", self.target_phrase()),
            Effect::RemoveFromCombat { .. } => format!("remove {} from combat", self.target_phrase()),
            Effect::Untap { .. } => format!("untap {}", self.target_phrase()),
            Effect::CounterSpell { .. }
            | Effect::CounterSpellDrawIfUnderpaid { .. }
            | Effect::CounterSpellToZone { .. }
            | Effect::CounterSpellExileNameLock { .. } => "counter target spell".into(),
            Effect::Fight { .. } => "fight".into(),
            Effect::DealDamageEqualToPower { .. } => "deal damage equal to power".into(),
            Effect::DealDamageEqualToPowerToEach { .. } => {
                "deal damage equal to power to each".into()
            }
            Effect::ExchangeControl { .. } | Effect::ExchangeControlChoosing { .. } => {
                "exchange control".into()
            }
            Effect::CreateToken { count, definition, .. } => {
                let n = match count {
                    Value::Const(n) => *n,
                    _ => 1,
                };
                let count_word = if n <= 1 { "a".to_string() } else { n.to_string() };
                let pt = if definition.card_types.contains(&crate::card::CardType::Creature) {
                    format!(" {}/{}", definition.power, definition.toughness)
                } else {
                    String::new()
                };
                let kw = if definition.keywords.is_empty() {
                    String::new()
                } else {
                    let words: Vec<String> = definition
                        .keywords
                        .iter()
                        .map(|k| format!("{k:?}").to_lowercase())
                        .collect();
                    format!(" with {}", words.join(", "))
                };
                let pluralised = if n > 1 && !definition.name.ends_with('s') {
                    format!("{} tokens", definition.name)
                } else {
                    definition.name.clone()
                };
                format!("create {count_word}{pt} {pluralised}{kw}")
            }
            Effect::GrantKeyword { keyword, .. } => {
                format!("grant {}", format!("{keyword:?}").to_lowercase())
            }
            Effect::GrantKeywords { keywords, .. } => {
                let names: Vec<String> =
                    keywords.iter().map(|k| format!("{k:?}").to_lowercase()).collect();
                format!("grant {}", names.join(", "))
            }
            Effect::Draw { amount, .. } => match amount {
                Value::Const(n) => {
                    if *n == 1 { "draw a card".into() } else { format!("draw {n} cards") }
                }
                _ => "draw cards".into(),
            },
            Effect::GainLife { amount, .. } => match amount {
                Value::Const(n) => format!("gain {n} life"),
                _ => "gain life".into(),
            },
            Effect::LoseLife { amount, .. } => match amount {
                Value::Const(n) => format!("lose {n} life"),
                _ => "lose life".into(),
            },
            Effect::Drain { amount, .. } => match amount {
                Value::Const(n) => format!("each opponent loses {n} life, you gain {n} life"),
                _ => "drain life".into(),
            },
            Effect::DoubleLife { .. } => "double target player's life total".into(),
            Effect::ShuffleSelfIntoLibrary => "shuffle this card into its owner's library".into(),
            Effect::Scry { amount, .. } => match amount {
                Value::Const(n) => format!("scry {n}"),
                _ => "scry".into(),
            },
            Effect::Surveil { amount, .. } => match amount {
                Value::Const(n) => format!("surveil {n}"),
                _ => "surveil".into(),
            },
            Effect::Mill { amount, .. } => match amount {
                Value::Const(n) => format!("mill {n}"),
                _ => "mill".into(),
            },
            Effect::MillUntilLands { .. } => "mill until lands".into(),
            Effect::MillTwoRepeatSharedColor { .. } => "mill two, maybe repeat".into(),
            Effect::ExileTopOfLibrary { amount, .. } => match amount {
                Value::Const(n) => format!("exile top {n} of library"),
                _ => "exile top of library".into(),
            },
            Effect::ExileTopMintPerChosenColor { .. } => {
                "exile top of library; a token per chosen-color card".into()
            }
            Effect::DestroyTargets { .. } => "destroy the chosen targets".into(),
            Effect::Champion { .. } => "champion a permanent".into(),
            Effect::ExileUpToNFromGraveyards { .. } => "exile cards from graveyards".into(),
            Effect::SpellTaxUntilYourNextTurn { amount, .. } => {
                format!("opponents' spells cost {{{amount}}} more until your next turn")
            }
            Effect::Discard { amount, .. } => match amount {
                Value::Const(1) => "discard a card".into(),
                Value::Const(n) => format!("discard {n} cards"),
                _ => "discard".into(),
            },
            Effect::Sacrifice { .. } => "sacrifice".into(),
            Effect::SacrificeSource => "sacrifice this".into(),
            Effect::ExileSource => "exile this".into(),
            Effect::Explore { .. } => "explore".into(),
            Effect::Goad { .. } => "goad target creature".into(),
            Effect::Suspect { .. } => "suspect target creature".into(),
            Effect::Discover { .. } => "discover".into(),
            Effect::ExileTopUntilNonlandMayPlay { free, .. } => {
                if *free {
                    "exile from the top of your library until a nonland card; you may cast it".into()
                } else {
                    "exile from the top of your library until a nonland card; you may play it this turn".into()
                }
            }
            // Walk every child and concatenate the non-empty pieces. The
            // earlier "first non-empty wins" version produced a misleading
            // summary for cards like Artistic Process mode 2 — Seq([
            // CreateToken, GrantKeyword]) returned the GrantKeyword text
            // alone (CreateToken had no arm), dropping the headline create
            // action.
            Effect::NameOpponentCastLock => {
                "opponents can't cast the named card until your next turn".into()
            }
            Effect::ResetCreature { power, toughness, creature_types, .. } => {
                let ty = creature_types
                    .first()
                    .map(|t| format!("{t:?}").to_lowercase())
                    .unwrap_or_else(|| "creature".into());
                match (power, toughness) {
                    (Value::Const(p), Value::Const(tn)) => {
                        format!("{} becomes a {p}/{tn} {ty}", self.target_phrase())
                    }
                    _ => format!("{} becomes a {ty}", self.target_phrase()),
                }
            }
            Effect::BecomeColor { colors, .. } => {
                let words: Vec<String> =
                    colors.iter().map(|c| format!("{c:?}").to_lowercase()).collect();
                format!("{} becomes {}", self.target_phrase(), words.join(" and "))
            }
            Effect::BecomeCreatureType { creature_types, .. }
            | Effect::AddCreatureTypes { creature_types, .. } => {
                let words: Vec<String> =
                    creature_types.iter().map(|t| format!("{t:?}")).collect();
                format!("{} becomes a {}", self.target_phrase(), words.join(" "))
            }
            Effect::LoseAllAbilities { .. } => {
                format!("{} loses all abilities", self.target_phrase())
            }
            Effect::SetBasePT { power, toughness, .. } => {
                let t = self.target_phrase();
                match (power, toughness) {
                    (Value::Const(p), Value::Const(tn)) => {
                        format!("{t} has base power and toughness {p}/{tn}")
                    }
                    _ => format!("{t} has a new base power and toughness"),
                }
            }
            Effect::Seq(v) => {
                let parts: Vec<String> = v
                    .iter()
                    .map(|e| e.effect_short_text())
                    .filter(|s| !s.is_empty())
                    .collect();
                parts.join(", then ")
            }
            Effect::If { then, else_, .. } => {
                let t = then.effect_short_text();
                if !t.is_empty() {
                    t
                } else {
                    else_.effect_short_text()
                }
            }
            Effect::MayDo { body, .. }
            | Effect::CapTargetsAtX { body }
            | Effect::MayPayX { body, .. }
            | Effect::MayPay { body, .. }
            | Effect::MayPayLife { body, .. }
            | Effect::DelayUntil { body, .. }
            | Effect::Repeat { body, .. }
            | Effect::Reflexive { body }
            | Effect::ForEach { body, .. } => body.effect_short_text(),
            Effect::Process { then, .. } => then.effect_short_text(),
            // Library tutor / dig — surfaced so modal pickers (Glimpse the Core,
            // the Confluence cycle) render a real label instead of a blank row.
            Effect::Search { to, .. } => {
                use crate::effect::ZoneDest;
                match to {
                    ZoneDest::Battlefield { .. } => "search your library for a card and put it onto the battlefield".into(),
                    ZoneDest::Hand(_) => "search your library for a card and put it into your hand".into(),
                    _ => "search your library for a card".into(),
                }
            }
            Effect::SearchLibraryOrGraveyard { .. } => {
                "search your library and/or graveyard for a card".into()
            }
            Effect::AddManaEqualToPermanentCost { .. } => {
                "add mana equal to the enchanted permanent's mana cost".into()
            }
            Effect::NameCardExileMatchingAllZones => {
                "name a card; exile every copy from that player's hand, graveyard, and library".into()
            }
            Effect::ChooseTypeRevealTopPartition { count } => match count {
                Value::Const(n) => format!(
                    "choose a card type, then reveal the top {n}; keep that type, bin the rest"),
                _ => "choose a card type, then reveal cards; keep that type, bin the rest".into(),
            },
            Effect::FertileImagination { per } => match per {
                Value::Const(n) => format!(
                    "choose a card type; make {n} Saprolings per matching card in target hand"),
                _ => "choose a card type; make Saprolings per matching card in target hand".into(),
            },
            Effect::GuildFeud => {
                "each side reveals three, deploys a creature; the two deployed creatures fight".into()
            }
            Effect::InfernalTutor => {
                "reveal a hand card and tutor its twin (Hellbent: tutor any card)".into()
            }
            Effect::IgnorantBliss => {
                "exile your hand face down; return it and draw at the next end step".into()
            }
            Effect::Dovescape => {
                "counter that noncreature spell; its caster makes a Bird per mana value".into()
            }
            Effect::IsperiaReveal => {
                "name a card; if the defender reveals it, tutor a flying creature".into()
            }
            Effect::GraveBetrayalRegister => {
                "reanimate that creature under your control at the next end step".into()
            }
            Effect::GraveBetrayalReanimate => "reanimate the fallen creature".into(),
            Effect::KindleTheCarnage => {
                "discard at random → deal its mana value to each creature, repeatable".into()
            }
            Effect::ChooseTwoColorsForSource => "choose two colors".into(),
            Effect::GainLifePerChosenColorOfCast => {
                "gain 1 life per chosen color the cast spell is".into()
            }
            Effect::AethermagesTouch { count } => match count {
                Value::Const(n) => format!(
                    "reveal the top {n}; put a creature onto the battlefield until your end step, bottom the rest"),
                _ => "reveal cards; deploy a creature until your end step, bottom the rest".into(),
            },
            _ => String::new(),
        }
    }

    /// True if a `Target::Player(_)` is a meaningful primary target for this
    /// effect. The auto-target heuristic uses this to skip player candidates
    /// when the effect actually operates on permanents — without it, an
    /// `Any`-filtered Move (Regrowth) auto-targets the caster as a player and
    /// silently fizzles, since `Effect::Move` only consumes
    /// `EntityRef::{Permanent,Card}` and ignores Player entries.
    ///
    /// Returns true for effects that legitimately point at a player face:
    /// damage, life-gain/loss, drain, mill/draw/discard against a player ref,
    /// surveil/scry/look (no-op for non-player anyway). False for effects that
    /// move/tap/destroy/exile cards.
    pub fn accepts_player_target(&self) -> bool {
        match self {
            Effect::DealDamage { .. }
            | Effect::GainLife { .. }
            | Effect::LoseLife { .. }
            | Effect::SetLifeTotal { .. }
            | Effect::Drain { .. }
            | Effect::Discard { .. }
            | Effect::DiscardAnyNumber { .. }
            | Effect::SetNoMaxHandSize { .. }
            | Effect::SetMaxHandSize { .. }
            | Effect::Draw { .. }
            | Effect::Mill { .. }
            | Effect::MillUntilLands { .. }
            | Effect::MillTwoRepeatSharedColor { .. }
            | Effect::ExileTopOfLibrary { .. }
            | Effect::ExileTopMintPerChosenColor { .. }
            | Effect::MillHalf { .. }
            | Effect::DiscardHalf { .. }
            | Effect::SacrificeHalf { .. }
            | Effect::AddPoison { .. } => true,
            // Cross-library searches target the searched player.
            Effect::SearchPickedBy { who: PlayerRef::Target(_), .. } => true,
            Effect::Search { who: PlayerRef::Target(_), .. } => true,
            // Divided damage allows player targets only when its filter can
            // match a player (Crackle with Power "any target"); creature-only
            // divide spells (Forked Bolt, Pyrokinesis) reject players.
            Effect::DealDamageDivided { filter, .. }
            | Effect::DealDamageDividedEvenly { filter, .. } => filter.can_match_player(),
            Effect::ApplyToTargets { filter, .. } => filter.can_match_player(),
            Effect::DeliverUntoEvil { filter, .. } => filter.can_match_player(),
            // Support / distribute put counters on creatures only — never players.
            Effect::SupportCounters { .. } => false,
            Effect::DistributeCounters { .. } => false,
            // Stack-targeted counter spells take a permanent slot but the
            // target is a stack item, not a player. Reject player target.
            Effect::CounterSpell { .. }
            | Effect::CounterSpellDrawIfUnderpaid { .. }
            | Effect::CounterSpellToZone { .. }
            | Effect::CounterSpellExileNameLock { .. }
            | Effect::CounterAbility { .. }
            | Effect::CounterSpellOrAbility { .. }
            | Effect::CounterUnlessPaid { .. }
            | Effect::CounterUnless { .. }
            | Effect::MakeSpellUncounterable { .. } => false,
            Effect::UnlessPlayerPays { then, .. } => then.accepts_player_target(),
            // "Gain control of all creatures target PLAYER controls"
            // (Emrakul, the World Anew) takes a player; the plain
            // permanent-steal form doesn't.
            Effect::GainControl { what, .. } => {
                matches!(what, Selector::ControlledBy { who: PlayerRef::Target(_), .. })
            }
            // Targets a card to recast (graveyard/exile), not a player.
            Effect::CastWithoutPayingImmediate { .. } => false,
            // "Tap all lands target player controls" takes a player
            // (Mistbind Clique); the plain selector forms don't.
            Effect::PhaseOut { what, .. }
            | Effect::GrantSuspend { what, .. }
            | Effect::ModularCounters { what }
            | Effect::Tap { what }
            | Effect::SetSaddled { what }
            | Effect::TapAndUntapLock { what }
            | Effect::Untap { what, .. } => {
                matches!(what, Selector::ControlledBy { who: PlayerRef::Target(_), .. })
            }
            // Permanent-targeting effects: skip Player.
            Effect::Destroy { .. }
            | Effect::DestroyAndRemember { .. }
            | Effect::DestroyNoRegen { .. }
            | Effect::Exile { .. }
            | Effect::Move { .. }
            | Effect::AddCounter { .. }
            | Effect::RemoveCounter { .. }
            | Effect::AddKeywordCounter { .. }
            | Effect::RemoveKeywordCounter { .. }
            | Effect::AddRandomMissingCounter { .. }
            | Effect::PumpPT { .. }
            | Effect::SetBasePT { .. }
            | Effect::SwitchPT { .. }
            | Effect::BecomeCreature { .. }
            | Effect::AnimateAsCreature { .. }
            | Effect::SetBasePower { .. }
            | Effect::GrantKeyword { .. }
            | Effect::GrantKeywords { .. }
            | Effect::ResetCreature { .. }
            | Effect::BecomeBasicLand { .. }
            | Effect::Attach { .. }
            | Effect::ExchangeControl { .. }
            | Effect::ExchangeControlChoosing { .. }
            | Effect::DealDamageEqualToPower { .. }
            | Effect::DealDamageEqualToPowerToEach { .. }
            | Effect::Fight { .. } => false,
            // Compound effects: defer to whichever child first surfaces a
            // primary-target filter — the auto-target heuristic's slot 0
            // is shared across the Seq, so a leading `Move(target → exile)`
            // dictates the target type for the whole spell, even if a
            // trailing `If(... GainLife)` would also accept Player. The
            // real-card example is Cling to Dust:
            //   `Seq([Move(target → Exile), If(EntityMatches Creature, GainLife)])`
            // Without this rule the bot picked Player(opp) first, which
            // matched the `Any` filter but silently fizzled at Move
            // resolution (Move only consumes Permanent/Card refs).
            Effect::Seq(v) => v
                .iter()
                .find(|e| e.primary_target_filter().is_some())
                .map(|e| e.accepts_player_target())
                .unwrap_or_else(|| v.iter().any(|e| e.accepts_player_target())),
            Effect::If { then, else_, .. } => {
                // Prefer the `then` branch (the active outcome) — same
                // logic as `ability_effect_label`. Fall back to else_'s
                // classification if `then` doesn't have a primary target.
                if then.primary_target_filter().is_some() {
                    then.accepts_player_target()
                } else if else_.primary_target_filter().is_some() {
                    else_.accepts_player_target()
                } else {
                    then.accepts_player_target() || else_.accepts_player_target()
                }
            }
            Effect::DelayUntil { body, .. }
            | Effect::Repeat { body, .. }
            | Effect::ForEach { body, .. } => body.accepts_player_target(),
            Effect::MayDo { body, .. }
            | Effect::CapTargetsAtX { body }
            | Effect::MayPayX { body, .. }
            | Effect::MayPay { body, .. }
            | Effect::MayPayLife { body, .. } => body.accepts_player_target(),
            Effect::Process { then, .. } => then.accepts_player_target(),
            Effect::ChooseMode(modes) => modes.iter().any(|e| e.accepts_player_target()),
            Effect::ChooseN { modes, .. } => modes.iter().any(|e| e.accepts_player_target()),
            Effect::FlipCoin { on_heads, on_tails, .. } => {
                on_heads.accepts_player_target() || on_tails.accepts_player_target()
            }
            Effect::FlipCoinsUntilLoseOrStop { tiers } => {
                tiers.iter().any(|(_, e)| e.accepts_player_target())
            }
            Effect::RollDie { results, .. } => {
                results.iter().any(|(_, _, e)| e.accepts_player_target())
            }
            // Conservative default: anything we don't classify is permitted.
            // The legality gate (filter + check_target_legality) still rejects
            // mismatched types, this just changes the heuristic's preference
            // order.
            _ => true,
        }
    }

    fn value_is_non_negative(v: &Value) -> bool {
        match v {
            Value::Const(n) => *n >= 0,
            // Dynamic values (`SacrificedPower`, `XFromCost`, etc.) are always
            // ≥ 0 in practice.
            _ => true,
        }
    }

    fn keyword_is_friendly(kw: &Keyword) -> bool {
        // Most keywords benefit the bearer, so a grant defaults to a friendly
        // pick. The exceptions are the "can't act" / restriction keywords —
        // a "target creature can't block" grant should hit an *opponent's*
        // creature, so classify those as hostile.
        !matches!(
            kw,
            Keyword::Defender
                | Keyword::Decayed
                | Keyword::CantBlock
                | Keyword::CantAttack
                | Keyword::CantAttackAlone
                | Keyword::CantAttackOrBlockAlone
        )
    }

    /// Walk the effect tree and return the first `SelectionRequirement` bound
    /// to the target slot `slot`, if any. Used for cast-time target validation.
    ///
    /// `mode` lets modal cards (`ChooseMode`) constrain the search to the
    /// chosen branch rather than picking up the first matching filter from
    /// any mode. Pass `None` for non-modal effects or to fall through to
    /// the legacy behaviour (first match across all modes).
    pub fn target_filter_for_slot_in_mode(
        &self,
        slot: u8,
        mode: Option<usize>,
    ) -> Option<&SelectionRequirement> {
        self.target_filter_for_slot_in_mode_kicked(slot, mode, false)
    }

    /// Kicker-aware variant: when `kicked`, an `If(SpellWasKicked, …)`
    /// resolves to its `then` branch's filter (and `else_`'s otherwise) so
    /// the cast-time target legality matches the branch that will resolve
    /// (Tear Asunder's kicked "nonland permanent" vs base "artifact or
    /// enchantment"). The non-kicked callers use the default-`false` wrapper.
    pub fn target_filter_for_slot_in_mode_kicked(
        &self,
        slot: u8,
        mode: Option<usize>,
        kicked: bool,
    ) -> Option<&SelectionRequirement> {
        fn sel_find(s: &Selector, slot: u8) -> Option<&SelectionRequirement> {
            // A `ControlledBy { who: Target(n) }` selector declares slot `n`
            // as a *player* target (How to Start a Riot's "creatures target
            // player controls get +2/+0"). Surface a Player filter so the
            // cast/auto-target walk prompts for that slot.
            const PLAYER: SelectionRequirement = SelectionRequirement::Player;
            match s {
                Selector::TargetFiltered { slot: s2, filter } if *s2 == slot => Some(filter),
                Selector::ControlledBy { who: PlayerRef::Target(s2), .. } if *s2 == slot => {
                    Some(&PLAYER)
                }
                // A bare `Player(Target(n))` selector declares slot `n` as a
                // player target — e.g. Lord of the Void's "exile the top seven
                // of that player's library" (`ExileTopOfLibrary { who:
                // Player(Target(0)) }`).
                Selector::Player(PlayerRef::Target(s2)) if *s2 == slot => Some(&PLAYER),
                Selector::AttachedTo(i)
                | Selector::AttachedToMe(i)
                | Selector::RadianceGroup { subject: i }
                | Selector::CreaturesInCombatWith(i)
                | Selector::SharingNameWith(i) => sel_find(i, slot),
                Selector::Take { inner, .. }
                | Selector::TakeRandom { inner, .. } => sel_find(inner, slot),
                Selector::TakeWithSumCap { inner, .. } => sel_find(inner, slot),
                _ => None,
            }
        }
        // A target slot can hide inside a `Value` sub-tree — Rabid Bite
        // deals damage equal to `Value::PowerOf(TargetFiltered{slot:0})`.
        // Descend so slot 0's filter is discoverable for cast/auto-target.
        fn val_find(v: &Value, slot: u8) -> Option<&SelectionRequirement> {
            match v {
                Value::CountOf(s)
                | Value::PowerOf(s)
                | Value::ToughnessOf(s)
                | Value::ManaValueOf(s)
                | Value::LoyaltyOf(s) => sel_find(s, slot),
                Value::CountersOn { what, .. } => sel_find(what, slot),
                // Arithmetic combinators can wrap a target-bearing value
                // (Polliwallop: `Times(PowerOf(slot 0), 2)`). Descend both arms.
                Value::Times(a, b) | Value::Diff(a, b) | Value::Min(a, b) | Value::Max(a, b) => {
                    val_find(a, slot).or_else(|| val_find(b, slot))
                }
                Value::NonNeg(a) | Value::HalvedRoundUp(a) => val_find(a, slot),
                Value::Sum(vs) => vs.iter().find_map(|x| val_find(x, slot)),
                // Player-ref values carrying a bare `PlayerRef::Target(n)`
                // register a player slot (Channeled Force's "the chosen
                // opponent's hand size" — the opponent's ONLY mention is
                // inside the draw amount).
                Value::HandSizeOf(p)
                | Value::LifeOf(p)
                | Value::GraveyardSizeOf(p)
                | Value::LibrarySizeOf(p)
                | Value::PermanentCountControlledBy(p)
                | Value::CreatureCountControlledBy(p) => implicit_player_for_ref_slot(p, slot),
                _ => None,
            }
        }
        fn eff_find(
            e: &Effect,
            slot: u8,
            mode: Option<usize>,
            kicked: bool,
        ) -> Option<&SelectionRequirement> {
            match e {
                Effect::Seq(v) => v.iter().find_map(|x| eff_find(x, slot, mode, kicked)),
                // `If(SpellWasKicked, …)` chooses the branch that will
                // actually resolve so cast-time target legality matches it.
                Effect::If { cond: Predicate::SpellWasKicked, then, else_ } => {
                    if kicked {
                        eff_find(then, slot, mode, kicked)
                    } else {
                        eff_find(else_, slot, mode, kicked)
                    }
                }
                Effect::If { then, else_, .. } => eff_find(then, slot, mode, kicked)
                    .or_else(|| eff_find(else_, slot, mode, kicked)),
                // A death-watch that is its slot's only mention declares the
                // filter itself (Melira's "another target creature or artifact").
                Effect::WhenTargetDiesThisTurn { filter: Some(f), slot: s, .. }
                    if *s as u8 == slot => Some(f),
                Effect::GainLifeWhenTargetDealsDamageThisTurn { slot: s }
                    if *s as u8 == slot => Some(&SelectionRequirement::Creature),
                Effect::ForEach { selector, body } => {
                    sel_find(selector, slot).or_else(|| eff_find(body, slot, mode, kicked))
                }
                Effect::Repeat { body, .. } => eff_find(body, slot, mode, kicked),
                Effect::ChooseMode(modes) => match mode {
                    // Mode-aware path: only look in the chosen branch.
                    Some(m) if m < modes.len() => eff_find(&modes[m], slot, None, kicked),
                    // Legacy path: first hit across all modes.
                    _ => modes.iter().find_map(|m| eff_find(m, slot, None, kicked)),
                },
                // ChooseN: each target-bearing picked mode occupies one
                // cast-time slot in pick order — slot 0 = the first picked
                // mode that needs a target, slot 1 = the second, etc. This
                // mirrors the resolution-time slot assignment so a "choose
                // one or both" spell (Steal the Show) can take a player
                // target for one mode and a creature target for the other.
                Effect::ChooseN { picks, modes } => {
                    let mut s = 0u8;
                    for &i in picks {
                        if let Some(m) = modes.get(i as usize)
                            && m.requires_target()
                        {
                            if s == slot {
                                return eff_find(m, 0, None, kicked);
                            }
                            s += 1;
                        }
                    }
                    // The definition's `picks` are only a default — real
                    // picks may be deferred to resolution. Fall back to the
                    // first mode that surfaces the slot so a submitted
                    // target is validated against *some* filter instead of
                    // none (Confluence cycle, Defend the Campus).
                    modes
                        .iter()
                        .find_map(|m| eff_find(m, slot.saturating_sub(s), None, kicked))
                }
                // Escalate: cast-time slot 0 validates against the base mode
                // (the cast-time `mode`), mirroring ChooseMode. Additional
                // escalate modes are chosen at resolution, so their later
                // slots aren't cast-validated (same approximation as ChooseN).
                Effect::Escalate { modes, .. } => match mode {
                    Some(m) if m < modes.len() => eff_find(&modes[m], slot, None, kicked),
                    _ => modes.iter().find_map(|m| eff_find(m, slot, None, kicked)),
                },
                // Spree targets are supplied and validated per chosen mode at
                // resolution (the chosen modes aren't encoded in the single
                // `mode` field, so a mode-agnostic slot filter would reject a
                // single-mode cast of any non-first mode). No cast-time slot
                // filter is surfaced; see `Effect::Spree`'s resolution arm.
                Effect::Spree { .. } | Effect::Tiered { .. } => None,
                // ChooseModesCast: the plain single-mode cast path
                // (`CastSpell { mode }`) validates like ChooseMode; a
                // multi-mode `CastSpellSpree` cast validates its per-instance
                // targets at resolution (falling back to the first mode that
                // surfaces the slot, mirroring ChooseN).
                Effect::ChooseModesCast { modes, .. } => match mode {
                    Some(m) if m < modes.len() => eff_find(&modes[m], slot, None, kicked),
                    _ => modes.iter().find_map(|m| eff_find(m, slot, None, kicked)),
                },
                Effect::MayDo { body, .. }
                | Effect::CapTargetsAtX { body }
                | Effect::MayPayX { body, .. }
                | Effect::MayPay { body, .. }
                | Effect::MayPayLife { body, .. } => eff_find(body, slot, mode, kicked),
                Effect::CollectEvidence { then, .. }
                | Effect::CollectEvidenceX { then } => eff_find(then, slot, mode, kicked),
                Effect::IfRevealFromHand { then, else_, .. } => {
                    eff_find(then, slot, mode, kicked).or_else(|| eff_find(else_, slot, mode, kicked))
                }
                Effect::FlipCoin { on_heads, on_tails, .. } => {
                    eff_find(on_heads, slot, mode, kicked).or_else(|| eff_find(on_tails, slot, mode, kicked))
                }
                Effect::FlipCoinsUntilLoseOrStop { tiers } => tiers
                    .iter()
                    .find_map(|(_, e)| eff_find(e, slot, mode, kicked)),
                Effect::RollDie { results, .. } => results
                    .iter()
                    .find_map(|(_, _, e)| eff_find(e, slot, mode, kicked)),
                Effect::DealDamage { to, amount }
                | Effect::EachControlledCreatureDealsDamage { to, amount }
                | Effect::DealDamageExcessToController { to, amount } => {
                    sel_find(to, slot).or_else(|| val_find(amount, slot))
                }
                Effect::RadianceDamage { subject, amount }
                | Effect::SameNameDamage { subject, amount } => {
                    sel_find(subject, slot).or_else(|| val_find(amount, slot))
                }
                Effect::PayAnyEnergyDealDamage { to } => sel_find(to, slot),
                // Each of slots 0..max_targets carries the divide filter, so
                // the cast/auto-target machinery collects "up to N targets".
                Effect::DealDamageDivided { filter, max_targets, .. }
                | Effect::DealDamageDividedEvenly { filter, max_targets, .. }
                | Effect::DistributeCounters { filter, max_targets, .. } => {
                    if slot < *max_targets { Some(filter) } else { None }
                }
                // X targets — every slot carries the filter.
                Effect::DestroyTargetsPolymorph { filter } => Some(filter),
                Effect::SupportCounters { filter, max_targets } => {
                    if slot < *max_targets { Some(filter) } else { None }
                }
                Effect::ApplyToTargets { filter, max_targets, .. }
                | Effect::DeliverUntoEvil { filter, max_targets, .. } => {
                    if slot < *max_targets { Some(filter) } else { None }
                }
                // Finale of Promise — slot 0 instant, slot 1 sorcery, each a
                // graveyard card of mana value X or less.
                Effect::FinaleOfPromise => match slot {
                    0 => Some(&FINALE_INSTANT_SLOT),
                    1 => Some(&FINALE_SORCERY_SLOT),
                    _ => None,
                },
                Effect::PreventNextDamage { target, .. }
                | Effect::PreventNextDamageAndGainLife { target, .. }
                | Effect::PreventAllDamageThisTurn { target }
                | Effect::ReplaceNextDamageWithDestroy { target }
                | Effect::PreventAllCombatDamageInvolving { target }
                | Effect::PreventCombatDamageToTargetThisTurn { target }
                | Effect::PreventCombatDamageByTargetThisTurn { target } => sel_find(target, slot),
                Effect::Fight { attacker, defender } => {
                    sel_find(attacker, slot).or_else(|| sel_find(defender, slot))
                }
                Effect::DealDamageEqualToPower { source, target } => {
                    sel_find(source, slot).or_else(|| sel_find(target, slot))
                }
                Effect::DealDamageEqualToPowerToEach { source, targets, .. } => {
                    sel_find(source, slot).or_else(|| sel_find(targets, slot))
                }
                Effect::ExchangeControl { a, b } => {
                    sel_find(a, slot).or_else(|| sel_find(b, slot))
                }
                Effect::RedirectNextDamage { target, to, .. } => {
                    sel_find(target, slot).or_else(|| sel_find(to, slot))
                }
                Effect::ExchangeControlChoosing { with, .. } => sel_find(with, slot),
                // `amount` may read a target's power (Soul's Grace gains life
                // equal to target creature's power).
                Effect::GainLife { who, amount } | Effect::LoseLife { who, amount } => {
                    sel_find(who, slot).or_else(|| val_find(amount, slot))
                }
                Effect::LoseHalfLife { who, .. }
                | Effect::MillHalf { who, .. }
                | Effect::MillThenDrawPerType { who, .. }
                | Effect::DiscardHalf { who, .. }
                | Effect::SacrificeHalf { who, .. } => sel_find(who, slot),
                Effect::SetLifeTotal { who, .. } => sel_find(who, slot),
                Effect::Drain { from, to, .. } => sel_find(from, slot).or_else(|| sel_find(to, slot)),
                // `amount` may read a target's power/toughness (Soul's Majesty
                // draws equal to target creature's power).
                Effect::Draw { who, amount }
                | Effect::Mill { who, amount }
                | Effect::ExileTopOfLibrary { who, amount, .. } => {
                    sel_find(who, slot).or_else(|| val_find(amount, slot))
                }
                Effect::MillUntilLands { who, .. }
                | Effect::MillTwoRepeatSharedColor { who } => sel_find(who, slot),
                Effect::ExileTopMintPerChosenColor { who, .. } => sel_find(who, slot)
                    .or_else(|| implicit_player_for_bare_player_slot(who, slot)),
                Effect::DestroyTargets { filter } => Some(filter),
                Effect::Discard { who, .. } => sel_find(who, slot),
                Effect::DiscardAnyNumber { who } => sel_find(who, slot),
                Effect::DiscardChosen { from, .. } => sel_find(from, slot),
                Effect::BottomChosenFromHandAndDraw { from, .. } => sel_find(from, slot),
                Effect::SearchSplitOpponentChooses { opponent, .. } => sel_find(opponent, slot),
                Effect::RedirectSpellTargetToSelf { what } => sel_find(what, slot),
                Effect::RedirectYourDamageToChosen { what } => sel_find(what, slot),
                Effect::ManaClash { opponent } => sel_find(opponent, slot),
                Effect::SetNoMaxHandSize { who } => sel_find(who, slot),
                Effect::SetMaxHandSize { who, .. } => sel_find(who, slot),
                Effect::Move { what, .. }
                | Effect::MoveChosen { from: what, .. } => sel_find(what, slot),
                Effect::Destroy { what }
                | Effect::DestroyAndRemember { what }
                | Effect::DestroyNoRegen { what }
                    | Effect::ExileIfWouldDieThisTurn { what }
                | Effect::GrantFlashbackThisTurn { what }
                | Effect::GrantMiracle { what, .. }
                | Effect::Exile { what }
                | Effect::CounterSpell { what }
                | Effect::CounterSpellExileSameNamed { what }
                | Effect::CounterSpellDrawIfUnderpaid { what }
                | Effect::CounterSpellToZone { what, .. }
                | Effect::CounterSpellExileNameLock { what }
                | Effect::CounterAbility { what }
            | Effect::CounterSpellOrAbility { what }
                | Effect::CounterUnlessPaid { what, .. }
                | Effect::CounterUnless { what, .. }
                | Effect::MakeSpellUncounterable { what }
                | Effect::Suspect { what }
                | Effect::ClearSuspected { what }
                | Effect::GainControl { what, .. }
                | Effect::GainControlWhileSourceRemains { what }
                | Effect::WeldArtifacts { what } => sel_find(what, slot),
                Effect::UnlessPlayerPays { then, .. } => eff_find(then, slot, mode, kicked),
                Effect::ExilePlayerGraveyard { who }
                | Effect::ExileHand { who }
                | Effect::ShuffleGraveyardCardsIntoLibrary { who, .. }
                | Effect::DiscardUnlessKind { who, .. } => implicit_player_for_ref_slot(who, slot),
                Effect::PhaseOut { what, .. }
                | Effect::GrantSuspend { what, .. }
                | Effect::ModularCounters { what }
                | Effect::Tap { what }
                | Effect::SetSaddled { what }
                | Effect::TapAndUntapLock { what }
                | Effect::Untap { what, .. } => {
                    sel_find(what, slot).or_else(|| implicit_player_for_slot(what, slot))
                }
                Effect::PumpPT { what, .. }
                | Effect::SetBasePT { what, .. }
                | Effect::SwitchPT { what, .. }
                | Effect::DoublePower { what, .. } => {
                    sel_find(what, slot).or_else(|| implicit_creature_for_slot(what, slot))
                }
                Effect::BecomeCreature { what, .. } => sel_find(what, slot),
                Effect::AnimateAsCreature { what, .. } => sel_find(what, slot),
                Effect::SetBasePower { what, .. } => sel_find(what, slot),
                Effect::GrantKeyword { what, .. }
                | Effect::GrantKeywords { what, .. }
                | Effect::GrantProtectionFromChosenColor { what, .. } => sel_find(what, slot),
                Effect::AddCounter { what, .. } | Effect::RemoveCounter { what, .. } => {
                    sel_find(what, slot)
                }
                Effect::RemoveAllCounters { what } | Effect::RemoveAnyCounter { what } => sel_find(what, slot),
                Effect::RemoveCountersUpTo { what, .. } => sel_find(what, slot),
                Effect::AddKeywordCounter { what, .. }
                | Effect::RemoveKeywordCounter { what, .. }
                | Effect::AddRandomMissingCounter { what, .. } => sel_find(what, slot),
                Effect::BecomeBasicLand { what, .. }
                | Effect::ResetCreature { what, .. } => sel_find(what, slot),
                Effect::RevealUntilLandDamage { to, .. }
                | Effect::RevealUntilNonlandDamage { to } => sel_find(to, slot),
                Effect::Attach { what, to } => sel_find(what, slot).or_else(|| sel_find(to, slot)),
                Effect::CreateTokenAttachedTo { target, .. }
                | Effect::CreateTokenAttachedToEach { target, .. } => sel_find(target, slot),
                Effect::CopySpell { what, .. }
                | Effect::CopySpellWithRiders { what, .. }
                | Effect::CopySpellMayChooseTargets { what, .. }
                | Effect::CopySpellUnlessPaid { what, .. }
                | Effect::ChooseNewTargetsForSpell { what } => sel_find(what, slot),
                Effect::Sacrifice { who, .. } | Effect::SacrificeGreatestMV { who, .. } => {
                    sel_find(who, slot)
                }
                Effect::AddPoison { who, .. } => sel_find(who, slot),
                // Single-selector targeted effects (cast-time filter
                // enforcement, CR 115.1a/601.2c). Keep in sync with
                // `requires_target` / `primary_target_filter` — the
                // `targeted_effects_carry_slot_filters` test guards this set.
                Effect::Regenerate { what }
                | Effect::ExileWithSource { what }
                | Effect::ExileSameNameAsTarget { what }
                | Effect::ExileTaggedWithSource { what }
                | Effect::ExileUntilSourceLeaves { what, .. }
                | Effect::ExileReturnToOwnerNextEndStep { what }
                | Effect::ExileReturnNextEndStep { what }
                | Effect::FlipUntilLossThenTokenCopies { what }
                | Effect::RemoveAllCountersDiscountNextSpell { what }
                | Effect::Goad { what }
                | Effect::Detain { what }
                | Effect::Provoke { what }
                | Effect::MustBlockSource { what }
                | Effect::Transform { what }
                | Effect::Flip { what }
                | Effect::LoseAllAbilities { what, .. }
                | Effect::LoseKeywordThisTurn { what, .. }
                | Effect::SkipNextUntap { what }
                | Effect::GrantTriggeredAbility { what, .. }
                | Effect::GainActivatedAbility { what, .. }
                | Effect::AddCardTypeIndefinitely { what, .. }
                | Effect::SetLoyalty { what, .. }
                | Effect::GrantLoyaltyTwiceThisTurn { what }
                | Effect::BecomeTreasure { what }
                | Effect::AddCounterOfPresentKind { what }
                | Effect::BecomeChosenColor { what, .. }
                | Effect::BecomeColor { what, .. }
                | Effect::BecomeCreatureType { what, .. }
                | Effect::AddCreatureTypes { what, .. }
                | Effect::ReplaceColorWord { what, .. }
                | Effect::ReplaceBasicLandType { what, .. }
                | Effect::GrantMayPlay { what, .. }
                | Effect::CastWithoutPayingImmediate { what, .. }
                | Effect::DoubleCountersOnEach { what, .. }
                | Effect::DoubleAllCountersOn { what }
                | Effect::NameCreatureType { what }
                | Effect::NameCard { what }
                | Effect::LockTargetNameUntilYourNextTurn { what }
                | Effect::Explore { who: what } => sel_find(what, slot),
                Effect::CantBlockSourceThisTurn { target } => sel_find(target, slot),
                Effect::MoveAllCounters { from, to }
                | Effect::MoveCounter { from, to, .. } => {
                    sel_find(from, slot).or_else(|| sel_find(to, slot))
                }
                Effect::BecomeCopyOf { what, source, .. }
                | Effect::BecomeCopyOfFor { what, source, .. } => {
                    sel_find(what, slot).or_else(|| sel_find(source, slot))
                }
                // The token RECEIVER ("target player creates ...",
                // Echocasting Symposium / Emeritus of Truce) is a player
                // slot alongside the copied source's own slot.
                Effect::CreateTokenCopyOf { who, source, .. } => {
                    sel_find(source, slot).or_else(|| implicit_player_for_ref_slot(who, slot))
                }
                Effect::CreateToken { who, .. } => implicit_player_for_ref_slot(who, slot),
                Effect::CreateTokenCopiesHasteSac { source, .. } => sel_find(source, slot),
                Effect::Endure { target, .. } => sel_find(target, slot),
                Effect::Airbend { what } => sel_find(what, slot),
                Effect::LifeGainLockThisTurn { who }
                | Effect::LifeLockThisTurn { who }
                | Effect::GrantSpellsUncounterableThisTurn { who }
                | Effect::GrantCreatureSpellsUncounterableThisTurn { who }
                | Effect::GrantHexproofFromColorThisTurn { who, .. }
                | Effect::CantCastNoncreatureThisTurn { who } => sel_find(who, slot),
                Effect::ExchangeLifeTotals { a, b } => {
                    sel_find(a, slot).or_else(|| sel_find(b, slot))
                }
                Effect::DoubleLife { who } => sel_find(who, slot),
                // Wrappers that defer their target to an inner body.
                Effect::Forage { then } | Effect::Process { then, .. } => {
                    eff_find(then, slot, mode, kicked)
                }
                Effect::WithSacrificedPt { body, .. }
                | Effect::WithTappedPower { body, .. }
                | Effect::OnYourNextSpellCastThisTurn { body }
                | Effect::OnYourNextInstantSorceryThisTurn { body }
                | Effect::OnYourNextNamedSpellThisTurn { body }
                | Effect::OptionalTargets { body, .. }
                | Effect::DelayUntil { body, .. } => eff_find(body, slot, mode, kicked),
                Effect::PayEnergy { then, .. } | Effect::PayEnergyValue { then, .. } | Effect::PayAnyEnergy { then } => eff_find(then, slot, mode, kicked),
                Effect::PayEnergyOrElse { otherwise, .. }
                | Effect::PayEnergyOrElseValue { otherwise, .. }
                | Effect::PayManaOrElse { otherwise, .. } => {
                    eff_find(otherwise, slot, mode, kicked)
                }
                Effect::Tribute { otherwise, .. } => eff_find(otherwise, slot, mode, kicked),
                Effect::TemptingOffer { body } => eff_find(body, slot, mode, kicked),
                Effect::PlayersMayAccept { otherwise, .. } => {
                    eff_find(otherwise, slot, mode, kicked)
                }
                Effect::Punisher { options, otherwise, .. } => options
                    .iter()
                    .find_map(|e| eff_find(e, slot, mode, kicked))
                    .or_else(|| eff_find(otherwise, slot, mode, kicked)),
                Effect::NthResolutionThisTurn { branches } => branches
                    .iter()
                    .find_map(|e| eff_find(e, slot, mode, kicked)),
                Effect::SacrificeAnyNumber { per_each, .. } => {
                    eff_find(per_each, slot, mode, kicked)
                }
                _ => None,
            }
        }
        eff_find(self, slot, mode, kicked)
    }

    /// Mode-agnostic shorthand for `target_filter_for_slot_in_mode(slot, None)`.
    /// For modal effects, returns the first filter from any mode (legacy
    /// behaviour preserved for callers that don't yet thread mode info).
    pub fn target_filter_for_slot(&self, slot: u8) -> Option<&SelectionRequirement> {
        self.target_filter_for_slot_in_mode(slot, None)
    }

    /// The number of targets the effect's multi-target instance *requires*
    /// (`ApplyToTargets.min_targets`), or `None` when the slot-bearing
    /// effect is not an `ApplyToTargets` (conventional effects keep their
    /// mandatory slot 0). Slot `n` is an optional pick iff
    /// `n >= min_targets_in_mode(mode).unwrap_or(u8::MAX)` — i.e. every
    /// slot of an "up to N" (`min_targets: 0`) shape may be declined,
    /// while "one or two targets" (`min_targets: 1`) requires slot 0.
    /// Walks `Seq`/modal/may wrappers like `distinct_target_count`.
    pub fn min_targets_in_mode(&self, mode: Option<usize>) -> Option<u8> {
        match self {
            Effect::ApplyToTargets { min_targets, .. } => Some(*min_targets),
            // "up to four target cards" — every slot optional.
            Effect::DeliverUntoEvil { .. } => Some(0),
            // "up to one instant and/or up to one sorcery" — both optional.
            Effect::FinaleOfPromise => Some(0),
            Effect::OptionalTargets { min, .. } => Some(*min),
            Effect::Seq(v) => v.iter().find_map(|e| e.min_targets_in_mode(None)),
            Effect::ChooseMode(modes) => match mode {
                Some(m) => modes.get(m).and_then(|e| e.min_targets_in_mode(None)),
                None => modes.iter().find_map(|e| e.min_targets_in_mode(None)),
            },
            Effect::MayDo { body, .. }
            | Effect::CapTargetsAtX { body }
            | Effect::MayPayX { body, .. }
            | Effect::MayPay { body, .. }
            | Effect::MayPayLife { body, .. } => body.min_targets_in_mode(mode),
            _ => None,
        }
    }

    /// True when the effect's slot-`slot` target pick may be declined by
    /// the chooser ("up to N targets" / "any number of targets").
    pub fn target_slot_optional(&self, slot: u8, mode: Option<usize>) -> bool {
        self.min_targets_in_mode(mode)
            .is_some_and(|min| slot >= min)
    }

    /// CR 115.3 — the count of mutually-distinct targets a *single* multi-target
    /// instance consumes (the "up to / any number of / N target …" effects:
    /// `DealDamageDivided`, `SupportCounters`). Those N targets occupy slots
    /// `0..N` and must all differ. Returns `None` for effects whose `target`
    /// clauses are separate instances (a `Seq` of single-target effects), where
    /// the same object may legally fill each clause. Walks modal wrappers so a
    /// chosen mode's multi-target effect is found.
    pub fn distinct_target_count(&self, mode: Option<usize>) -> Option<u8> {
        match self {
            Effect::DealDamageDivided { max_targets, .. }
            | Effect::DealDamageDividedEvenly { max_targets, .. }
            | Effect::SupportCounters { max_targets, .. }
            | Effect::ApplyToTargets { max_targets, .. }
            | Effect::DeliverUntoEvil { max_targets, .. }
            | Effect::DistributeCounters { max_targets, .. } => Some(*max_targets),
            Effect::FinaleOfPromise => Some(2),
            Effect::ChooseMode(modes) => match mode {
                Some(m) => modes.get(m).and_then(|e| e.distinct_target_count(None)),
                None => modes.iter().find_map(|e| e.distinct_target_count(None)),
            },
            Effect::MayDo { body, .. }
            | Effect::CapTargetsAtX { body }
            | Effect::MayPayX { body, .. }
            | Effect::MayPay { body, .. }
            | Effect::MayPayLife { body, .. } => body.distinct_target_count(mode),
            _ => None,
        }
    }
}
