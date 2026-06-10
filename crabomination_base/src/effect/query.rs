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

/// Player restriction synthesized for the player slot referenced by a
/// `Selector::ControlledBy { who: PlayerRef::Target(n) }` — the spell targets
/// a player and then acts on the permanents that player controls (Sleep).
static IMPLICIT_PLAYER_TARGET: SelectionRequirement = SelectionRequirement::Player;

/// `Some(&Player)` when `what` is `ControlledBy { who: Target(n) }` for `slot`.
fn implicit_player_for_slot(what: &Selector, slot: u8) -> Option<&'static SelectionRequirement> {
    matches!(what, Selector::ControlledBy { who: PlayerRef::Target(n), .. } if *n == slot)
        .then_some(&IMPLICIT_PLAYER_TARGET)
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
                | Selector::SharingNameWith(i) => sel_has_target(i),
                Selector::Take { inner, count } => {
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
                Value::CountOf(s) | Value::PowerOf(s) | Value::ToughnessOf(s) => sel_has_target(s),
                Value::CountersOn { what, .. } => sel_has_target(what),
                Value::LifeOf(p) | Value::HandSizeOf(p) | Value::GraveyardSizeOf(p)
                | Value::LibrarySizeOf(p) => {
                    player_has_target(p)
                }
                Value::Sum(vs) => vs.iter().any(value_has_target),
                Value::Diff(a, b) | Value::Times(a, b) | Value::Min(a, b) | Value::Max(a, b) => {
                    value_has_target(a) || value_has_target(b)
                }
                Value::NonNeg(v) => value_has_target(v),
                Value::ManaValueOf(s) => sel_has_target(s),
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
            Effect::Noop => false,
            // Targets an opponent, but resolution auto-binds slot 0 / the
            // lowest-seat opponent, so no cast-time target is demanded.
            Effect::RevealOpponentTopPutOntoBattlefield { .. } => false,
            Effect::NameCardRevealTop { .. } => false,
            Effect::RevealTopToHandOpponentsLoseMv => false,
            Effect::PutFromHandOrGraveyardOntoBattlefield { .. } => false,
            Effect::StealCreatureEtbThisTurn => false,
            Effect::LookTopExileOneMayPlay { .. } => false,
            Effect::NameCardTargetDiscardsMatching => true,
            Effect::TemptingOffer { body } => body.requires_target(),
            // The accept branch's slot-0 player is bound at resolution; only
            // `otherwise` can demand a cast-time target (Browbeat's drawer).
            Effect::PlayersMayAccept { otherwise, .. } => otherwise.requires_target(),
            Effect::OnEachSpellCastThisTurn { .. } => false,
            Effect::PutExiledCreatureOntoBattlefield { .. } => false,
            Effect::ExileHand { who } => player_has_target(who),
            Effect::Demonstrate => false,
            Effect::Cipher => false,
            Effect::Myriad => false,
            Effect::Enlist => false,
            Effect::StudyTopCard { .. } => false,
            Effect::ExileTopWithCounters { .. } => false,
            Effect::HoneFromHand { .. } => false,
            Effect::PutFromHandOntoBattlefield { .. } => false,
            Effect::Manifest { .. } => false,
            Effect::ManifestDread { .. } => false,
            Effect::Cloak { .. } => false,
            Effect::CatchUpBasicLands => false,
            Effect::ExileFromHandTaxed { .. } => false,
            Effect::Hideaway { .. } => false,
            Effect::NthResolutionThisTurn { branches } => {
                branches.iter().any(|e| e.requires_target())
            }
            Effect::SacrificeSource => false,
            Effect::SacrificeSourceUnlessSacrifice { .. } => false,
            Effect::GrantNextInstantOrSorceryDiscountThisTurn { .. } => false,
            Effect::ReturnSelfAsEnchantment => false,
            Effect::Transform { what } => sel_has_target(what),
            Effect::Meld { .. } => false,
            Effect::SpellsCostLessThisTurn { .. } => false,
            Effect::CastFromHandWithoutPaying { .. } => false,
            Effect::PreventNextDamageFromChosenSource { .. } => false,
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
            Effect::RollDie { count, results, .. } => {
                value_has_target(count) || results.iter().any(|(_, _, e)| e.requires_target())
            }
            Effect::ChooseMode(modes) => modes.iter().any(|e| e.requires_target()),
            Effect::ChooseN { modes, .. } => modes.iter().any(|e| e.requires_target()),
            Effect::Escalate { modes, .. } => modes.iter().any(|e| e.requires_target()),
            Effect::MayDo { body, .. } => body.requires_target(),
            Effect::WithSacrificedPt { body, .. } => body.requires_target(),
            Effect::OnYourNextSpellCastThisTurn { body } => body.requires_target(),
            Effect::SearchSplitWithOpponent { .. } => false,
            Effect::ReturnResolvingSpellToHand => false,
            Effect::ExileResolvingSpell => false,
            Effect::SilencePlayersThisTurn { who } => player_has_target(who),
            Effect::MayPay { body, .. } => body.requires_target(),
            Effect::Process { then, .. } => then.requires_target(),
            Effect::CollectEvidence { amount, then } => {
                value_has_target(amount) || then.requires_target()
            }
            Effect::Forage { then } => then.requires_target(),
            Effect::Endure { target, n } => sel_has_target(target) || value_has_target(n),
            Effect::IfRevealFromHand { then, else_, .. } => {
                then.requires_target() || else_.requires_target()
            }
            Effect::DealDamage { to, amount } => sel_has_target(to) || value_has_target(amount),
            // Divided damage always targets (one or more chosen targets).
            Effect::DealDamageDivided { .. } => true,
            Effect::SupportCounters { .. } => true,
            Effect::Fight { attacker, defender } => {
                sel_has_target(attacker) || sel_has_target(defender)
            }
            Effect::ExchangeControl { a, b } => sel_has_target(a) || sel_has_target(b),
            Effect::GainLife { who, amount } | Effect::LoseLife { who, amount } => {
                sel_has_target(who) || value_has_target(amount)
            }
            Effect::LoseHalfLife { who, .. }
            | Effect::MillHalf { who, .. }
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
            Effect::Draw { who, amount }
            | Effect::Mill { who, amount }
            | Effect::ExileTopOfLibrary { who, amount, .. } => {
                sel_has_target(who) || value_has_target(amount)
            }
            Effect::Discard { who, amount, .. } => sel_has_target(who) || value_has_target(amount),
            Effect::DiscardAnyNumber { who } => sel_has_target(who),
            Effect::SetNoMaxHandSize { who } => sel_has_target(who),
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
            Effect::RevealTopTakeMatchingToHand { who, count, .. } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::ExileLibraryExceptBottom { who, keep } => {
                player_has_target(who) || value_has_target(keep)
            }
            Effect::Explore { who } => sel_has_target(who),
            Effect::Goad { what } => sel_has_target(what),
            Effect::Suspect { what } => sel_has_target(what),
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
            Effect::Search { who, to, .. } => player_has_target(who) || zonedest_has_target(to),
            Effect::ShuffleGraveyardIntoLibrary { who }
            | Effect::ShuffleHandAndGraveyardIntoLibrary { who } => player_has_target(who),
            Effect::ExchangeHandAndGraveyard { who } => player_has_target(who),
            Effect::ShuffleLibrary { who } => player_has_target(who),
            Effect::AddMana { who, pool } => {
                player_has_target(who) || match pool {
                    ManaPayload::Colorless(v)
                    | ManaPayload::AnyOneColor(v)
                    | ManaPayload::AnyColors(v) => value_has_target(v),
                    ManaPayload::OfColor(_, v) | ManaPayload::OfColors(_, v) => value_has_target(v),
                    ManaPayload::Restricted(inner, _)
                    | ManaPayload::RestrictedToChosenType(inner) => match inner.as_ref() {
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
                    | ManaPayload::AnyColorYouCouldProduce => false,
                }
            }
            Effect::Destroy { what }
            | Effect::DestroyNoRegen { what }
            | Effect::Regenerate { what }
            | Effect::ExileIfWouldDieThisTurn { what }
            | Effect::GrantFlashbackThisTurn { what }
            | Effect::GrantMiracle { what, .. }
            | Effect::Exile { what }
            | Effect::ExileWithSource { what }
            | Effect::RemoveAllCountersDiscountNextSpell { what }
            | Effect::ExileSameNameAsTarget { what }
            | Effect::ExileTaggedWithSource { what }
            | Effect::ExileUntilSourceLeaves { what, .. }
            | Effect::ExileReturnNextEndStep { what }
            | Effect::PhaseOut { what }
            | Effect::Tap { what }
            | Effect::Untap { what, .. }
            | Effect::Provoke { what }
            | Effect::CounterSpell { what }
            | Effect::CounterSpellToZone { what, .. }
            | Effect::CounterAbility { what }
            | Effect::CounterUnlessPaid { what, .. }
            | Effect::CounterUnless { what, .. } => sel_has_target(what),
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
            Effect::LoseKeywordThisTurn { what, .. } => sel_has_target(what),
            Effect::SacrificeAllMatching { who, .. } => sel_has_target(who),
            Effect::BecomeChosenColor { what, .. }
            | Effect::BecomeColor { what, .. }
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
            Effect::MoveCounter { from, to, amount, .. } => {
                sel_has_target(from) || sel_has_target(to) || value_has_target(amount)
            }
            Effect::RemoveAllCounters { what } => sel_has_target(what),
            Effect::SetLoyalty { what, value } => sel_has_target(what) || value_has_target(value),
            Effect::Proliferate => false,
            Effect::GainControl { what, .. } => sel_has_target(what),
            Effect::CreateToken { who, count, .. }
            | Effect::CreateTokenAttacking { who, count, .. }
            | Effect::Amass { who, count, .. } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::BecomeBasicLand { what, .. }
            | Effect::ResetCreature { what, .. } => sel_has_target(what),
            Effect::BecomeCopyOf { what, source, .. }
            | Effect::BecomeCopyOfFor { what, source, .. } => {
                sel_has_target(what) || sel_has_target(source)
            }
            Effect::Attach { what, to } => sel_has_target(what) || sel_has_target(to),
            Effect::CopySpell { what, count }
            | Effect::CopySpellMayChooseTargets { what, count } => {
                sel_has_target(what) || value_has_target(count)
            }
            Effect::ChooseNewTargetsForSpell { what } => sel_has_target(what),
            Effect::CopySpellUnlessPaid { what, count, .. } => {
                sel_has_target(what) || value_has_target(count)
            }
            Effect::GrantMayPlay { what, .. } => sel_has_target(what),
            Effect::CastWithoutPayingImmediate { what, .. } => sel_has_target(what),
            Effect::RegisterParadigm | Effect::CastFreeParadigmCopy => false,
            Effect::Cascade { .. } => false,
            Effect::Sacrifice { who, count, .. } => sel_has_target(who) || value_has_target(count),
            Effect::PlayerExilesPermanents { count, .. } => value_has_target(count),
            Effect::SacrificeGreatestMV { who, count, .. } => {
                sel_has_target(who) || value_has_target(count)
            }
            Effect::Punisher { chooser, options, otherwise } => {
                sel_has_target(chooser)
                    || options.iter().any(|e| e.requires_target())
                    || otherwise.requires_target()
            }
            Effect::AddPoison { who, amount } => sel_has_target(who) || value_has_target(amount),
            Effect::AddRadCounters { who, amount } => {
                sel_has_target(who) || value_has_target(amount)
            }
            Effect::RevealTopAndDrawIf { who, .. }
            | Effect::RevealTopCard { who }
            | Effect::RevealTopLandToBattlefieldElseHand { who }
            | Effect::RevealTopPutPermanentMvElseHand { who, .. }
            | Effect::RevealTopPutPermanentOntoBattlefield { who } => {
                player_has_target(who)
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
            // Registers a turn-scoped delayed trigger; no cast-time target.
            Effect::CreaturesYouControlEnteringThisTurn { .. } => false,
            Effect::PayOrLoseGame { .. } => false,
            Effect::SacrificeAndRemember { .. } => false,
            Effect::SacrificeAnyNumber { per_each, .. } => per_each.requires_target(),
            Effect::PayLifeLookTake { .. } => false,
            Effect::ExileAnyNumberFromGraveyards { .. } => false,
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
            | Effect::ExileChosenUntilSourceLeaves { from, count, .. }
            | Effect::ExileChosenFromHand { from, count, .. } => {
                sel_has_target(from) || value_has_target(count)
            }
            Effect::NameCreatureType { what } => sel_has_target(what),
            Effect::NameCard { what } => sel_has_target(what),
            Effect::WinGame { who } | Effect::LoseGame { who } => player_has_target(who),
            Effect::SkipTurns { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::TakeExtraTurn { who, count } => {
                player_has_target(who) || value_has_target(count)
            }
            Effect::AdditionalCombatPhase { count }
            | Effect::AdditionalCombatPhaseAfterMain { count } => value_has_target(count),
            Effect::CreateEmblem { who, .. } => player_has_target(who),
            Effect::CreateTokenCopyOf { who, count, source, .. } => {
                player_has_target(who) || value_has_target(count) || sel_has_target(source)
            }
            Effect::GrantTriggeredAbility { what, .. } => sel_has_target(what),
            Effect::PreventAllCombatDamageThisTurn => false,
            Effect::PreventAllCombatDamageInvolving { target } => sel_has_target(target),
            Effect::CantBlockSourceThisTurn { target } => sel_has_target(target),
            Effect::PreventNextDamage { target, amount }
            | Effect::PreventNextDamageAndGainLife { target, amount } => {
                sel_has_target(target) || value_has_target(amount)
            }
            Effect::PreventAllDamageThisTurn { target } => sel_has_target(target),
            Effect::DamageCantBePreventedThisTurn => false,
            Effect::PlayerProtectionUntilNextTurn { .. } => false,
            Effect::WhenLastCreatedTokenLeaves { .. } => false,
            Effect::DiminishCreaturesExceptChosenType { power, toughness } => {
                value_has_target(power) || value_has_target(toughness)
            }
            Effect::LifeGainLockThisTurn { who } => sel_has_target(who),
            Effect::GrantSpellsUncounterableThisTurn { who } => sel_has_target(who),
            Effect::CantCastNoncreatureThisTurn { who } => sel_has_target(who),
            Effect::ExileTopAndGrantMayPlay { .. } => false,
            Effect::AddEnergy(amount) => value_has_target(amount),
            Effect::PayEnergy { then, .. } => then.requires_target(),
            Effect::PayEnergyOrElse { otherwise, .. } => otherwise.requires_target(),
            Effect::PayManaOrElse { otherwise, .. } => otherwise.requires_target(),
            Effect::ExileTopMayPayEnergyToCast { .. } => false,
            Effect::DoubleCountersOnEach { what, .. } => sel_has_target(what),
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
                Selector::Take { inner, .. } => sel_filter(inner),
                Selector::TakeWithSumCap { inner, .. } => sel_filter(inner),
                _ => None,
            }
        }
        match self {
            // Prefer the damage target's own filter; fall back to a filter
            // hidden in the damage amount (Rabid Bite: `PowerOf(slot 0)`).
            Effect::DealDamage { to, amount } => sel_filter(to).or_else(|| match amount {
                Value::CountOf(s) | Value::PowerOf(s) | Value::ToughnessOf(s) => sel_filter(s),
                _ => None,
            }),
            Effect::DealDamageDivided { filter, .. } => Some(filter),
            // Fight surfaces the *defender's* filter (the opp creature
            // we want to fight). The attacker is usually the friendly
            // already-on-bf source/target.
            Effect::Fight { defender, .. } => sel_filter(defender),
            Effect::ExchangeControl { a, .. } => sel_filter(a),
            Effect::GainLife { who, .. } | Effect::LoseLife { who, .. } => sel_filter(who),
            Effect::LoseHalfLife { who, .. }
            | Effect::MillHalf { who, .. }
            | Effect::DiscardHalf { who, .. }
            | Effect::DoubleLife { who }
            | Effect::SacrificeHalf { who, .. } => sel_filter(who),
            Effect::SetLifeTotal { who, .. } => sel_filter(who),
            Effect::Destroy { what }
            | Effect::DestroyNoRegen { what }
            | Effect::Regenerate { what }
            | Effect::ExileIfWouldDieThisTurn { what }
            | Effect::GrantFlashbackThisTurn { what }
            | Effect::GrantMiracle { what, .. }
            | Effect::Exile { what }
            | Effect::ExileWithSource { what }
            | Effect::RemoveAllCountersDiscountNextSpell { what }
            | Effect::ExileSameNameAsTarget { what }
            | Effect::ExileTaggedWithSource { what }
            | Effect::ExileUntilSourceLeaves { what, .. }
            | Effect::ExileReturnNextEndStep { what }
            | Effect::PhaseOut { what }
            | Effect::Tap { what }
            | Effect::Untap { what, .. }
            | Effect::Provoke { what }
            | Effect::Suspect { what }
            | Effect::Detain { what }
            | Effect::CounterSpell { what }
            | Effect::CounterSpellToZone { what, .. }
            | Effect::CounterAbility { what }
            | Effect::CounterUnlessPaid { what, .. }
            | Effect::CounterUnless { what, .. }
            | Effect::CastWithoutPayingImmediate { what, .. }
            | Effect::CopySpell { what, .. }
            | Effect::CopySpellMayChooseTargets { what, .. }
            | Effect::GainControl { what, .. } => sel_filter(what),
            Effect::UnlessPlayerPays { then, .. } => then.primary_target_filter(),
            Effect::AddCounter { what, .. }
            | Effect::RemoveCounter { what, .. }
            | Effect::RemoveAllCounters { what }
            | Effect::SetLoyalty { what, .. }
            | Effect::AddKeywordCounter { what, .. }
            | Effect::RemoveKeywordCounter { what, .. } => sel_filter(what),
            // CreateTokenCopyOf — the `source` is the targeted permanent to
            // copy (Esika's Chariot "copy target token you control").
            Effect::CreateTokenCopyOf { source, .. } => sel_filter(source),
            Effect::PumpPT { what, .. }
            | Effect::SetBasePT { what, .. }
            | Effect::SwitchPT { what, .. }
            | Effect::DoublePower { what, .. } => {
                sel_filter(what).or_else(|| implicit_creature_if_bare_target(what))
            }
            Effect::BecomeCreature { what, .. } => sel_filter(what),
            Effect::GrantKeyword { what, .. }
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
            | Effect::ExileTopOfLibrary { who, .. } => sel_filter(who),
            Effect::Drain { to, .. } => sel_filter(to),
            Effect::AddPoison { who, .. } => sel_filter(who),
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
            // The copy *source* is the targeted slot ("becomes a copy of
            // target land").
            Effect::BecomeCopyOf { source, .. }
            | Effect::BecomeCopyOfFor { source, .. } => sel_filter(source),
            Effect::WhenTargetDiesThisTurn { .. } => Some(&SelectionRequirement::Creature),
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
            Effect::MayDo { body, .. } => body.primary_target_filter(),
            Effect::MayPay { body, .. } => body.primary_target_filter(),
            Effect::PayEnergy { then, .. } => then.primary_target_filter(),
            Effect::Process { then, .. } => then.primary_target_filter(),
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
            // Doubling a life total is a gift — point Beacon of Immortality at
            // the caster, not the opponent.
            Effect::DoubleLife { .. } => true,
            // Copying "target token you control" is friendly (Esika's Chariot).
            Effect::CreateTokenCopyOf { .. } => true,
            Effect::GrantKeyword { keyword, .. } => Self::keyword_is_friendly(keyword),
            Effect::AddCounter { kind, .. } => matches!(kind, CounterType::PlusOnePlusOne),
            Effect::Seq(v) => v.iter().any(|e| e.prefers_friendly_target()),
            Effect::If { then, else_, .. } => {
                then.prefers_friendly_target() || else_.prefers_friendly_target()
            }
            Effect::DelayUntil { body, .. } | Effect::Repeat { body, .. } => {
                body.prefers_friendly_target()
            }
            Effect::ForEach { body, .. } | Effect::MayDo { body, .. } => {
                body.prefers_friendly_target()
            }
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
            | Effect::MayDo { body, .. } => body.prefers_graveyard_target(),
            Effect::Process { then, .. } => then.prefers_graveyard_target(),
            // Recasting a target card *from the graveyard* (Efreet Flamepainter,
            // The Dawning Archaic) wants the graveyard walked for the target.
            Effect::CastWithoutPayingImmediate { source_zone, .. } => {
                matches!(source_zone, crate::card::Zone::Graveyard)
            }
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
            Effect::Destroy { .. } => format!("destroy {}", self.target_phrase()),
            Effect::DestroyNoRegen { .. } => {
                format!("destroy {} (can't be regenerated)", self.target_phrase())
            }
            Effect::Exile { .. } => format!("exile {}", self.target_phrase()),
            Effect::ExileUntilSourceLeaves { .. } => {
                format!("exile {} until this leaves the battlefield", self.target_phrase())
            }
            Effect::DealDamage { amount, .. } => {
                let t = self.target_phrase();
                match amount {
                    Value::Const(n) => format!("deal {n} damage to {t}"),
                    _ => format!("deal damage to {t}"),
                }
            }
            Effect::DealDamageDivided { total, .. } => match total {
                Value::Const(n) => format!("deal {n} damage divided among targets"),
                _ => "deal damage divided among targets".into(),
            },
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
            Effect::Tap { .. } => format!("tap {}", self.target_phrase()),
            Effect::PhaseOut { .. } => format!("phase out {}", self.target_phrase()),
            Effect::Untap { .. } => format!("untap {}", self.target_phrase()),
            Effect::CounterSpell { .. } | Effect::CounterSpellToZone { .. } => {
                "counter target spell".into()
            }
            Effect::Fight { .. } => "fight".into(),
            Effect::ExchangeControl { .. } => "exchange control".into(),
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
            Effect::ExileTopOfLibrary { amount, .. } => match amount {
                Value::Const(n) => format!("exile top {n} of library"),
                _ => "exile top of library".into(),
            },
            Effect::Discard { amount, .. } => match amount {
                Value::Const(1) => "discard a card".into(),
                Value::Const(n) => format!("discard {n} cards"),
                _ => "discard".into(),
            },
            Effect::Sacrifice { .. } => "sacrifice".into(),
            Effect::SacrificeSource => "sacrifice this".into(),
            Effect::Explore { .. } => "explore".into(),
            Effect::Goad { .. } => "goad target creature".into(),
            Effect::Suspect { .. } => "suspect target creature".into(),
            Effect::Discover { .. } => "discover".into(),
            // Walk every child and concatenate the non-empty pieces. The
            // earlier "first non-empty wins" version produced a misleading
            // summary for cards like Artistic Process mode 2 — Seq([
            // CreateToken, GrantKeyword]) returned the GrantKeyword text
            // alone (CreateToken had no arm), dropping the headline create
            // action.
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
            | Effect::MayPay { body, .. }
            | Effect::DelayUntil { body, .. }
            | Effect::Repeat { body, .. }
            | Effect::ForEach { body, .. } => body.effect_short_text(),
            Effect::Process { then, .. } => then.effect_short_text(),
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
            | Effect::ExileTopOfLibrary { .. }
            | Effect::MillHalf { .. }
            | Effect::DiscardHalf { .. }
            | Effect::SacrificeHalf { .. }
            | Effect::AddPoison { .. } => true,
            // Divided damage allows player targets only when its filter can
            // match a player (Crackle with Power "any target"); creature-only
            // divide spells (Forked Bolt, Pyrokinesis) reject players.
            Effect::DealDamageDivided { filter, .. } => filter.can_match_player(),
            // Support puts +1/+1 counters on creatures only — never players.
            Effect::SupportCounters { .. } => false,
            // Stack-targeted counter spells take a permanent slot but the
            // target is a stack item, not a player. Reject player target.
            Effect::CounterSpell { .. }
            | Effect::CounterSpellToZone { .. }
            | Effect::CounterAbility { .. }
            | Effect::CounterUnlessPaid { .. }
            | Effect::CounterUnless { .. } => false,
            Effect::UnlessPlayerPays { then, .. } => then.accepts_player_target(),
            // Targets a card to recast (graveyard/exile), not a player.
            Effect::CastWithoutPayingImmediate { .. } => false,
            // Permanent-targeting effects: skip Player.
            Effect::Destroy { .. }
            | Effect::DestroyNoRegen { .. }
            | Effect::Exile { .. }
            | Effect::PhaseOut { .. }
            | Effect::Tap { .. }
            | Effect::Untap { .. }
            | Effect::Move { .. }
            | Effect::AddCounter { .. }
            | Effect::RemoveCounter { .. }
            | Effect::AddKeywordCounter { .. }
            | Effect::RemoveKeywordCounter { .. }
            | Effect::PumpPT { .. }
            | Effect::SetBasePT { .. }
            | Effect::SwitchPT { .. }
            | Effect::BecomeCreature { .. }
            | Effect::GrantKeyword { .. }
            | Effect::GainControl { .. }
            | Effect::ResetCreature { .. }
            | Effect::BecomeBasicLand { .. }
            | Effect::Attach { .. }
            | Effect::ExchangeControl { .. }
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
            Effect::MayDo { body, .. } | Effect::MayPay { body, .. } => body.accepts_player_target(),
            Effect::Process { then, .. } => then.accepts_player_target(),
            Effect::ChooseMode(modes) => modes.iter().any(|e| e.accepts_player_target()),
            Effect::ChooseN { modes, .. } => modes.iter().any(|e| e.accepts_player_target()),
            Effect::FlipCoin { on_heads, on_tails, .. } => {
                on_heads.accepts_player_target() || on_tails.accepts_player_target()
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
        // Defensive / offensive keywords benefit the controller. We exclude
        // negative-value keywords like Defender / "can't attack" if they ever
        // get added; for now every Keyword variant is a buff.
        !matches!(
            kw,
            Keyword::Defender // arguably a debuff in isolation
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
            match s {
                Selector::TargetFiltered { slot: s2, filter } if *s2 == slot => Some(filter),
                Selector::AttachedTo(i)
                | Selector::AttachedToMe(i)
                | Selector::SharingNameWith(i) => sel_find(i, slot),
                Selector::Take { inner, .. } => sel_find(inner, slot),
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
                    None
                }
                // Escalate: cast-time slot 0 validates against the base mode
                // (the cast-time `mode`), mirroring ChooseMode. Additional
                // escalate modes are chosen at resolution, so their later
                // slots aren't cast-validated (same approximation as ChooseN).
                Effect::Escalate { modes, .. } => match mode {
                    Some(m) if m < modes.len() => eff_find(&modes[m], slot, None, kicked),
                    _ => modes.iter().find_map(|m| eff_find(m, slot, None, kicked)),
                },
                Effect::MayDo { body, .. } | Effect::MayPay { body, .. } => {
                    eff_find(body, slot, mode, kicked)
                }
                Effect::CollectEvidence { then, .. } => eff_find(then, slot, mode, kicked),
                Effect::IfRevealFromHand { then, else_, .. } => {
                    eff_find(then, slot, mode, kicked).or_else(|| eff_find(else_, slot, mode, kicked))
                }
                Effect::FlipCoin { on_heads, on_tails, .. } => {
                    eff_find(on_heads, slot, mode, kicked).or_else(|| eff_find(on_tails, slot, mode, kicked))
                }
                Effect::RollDie { results, .. } => results
                    .iter()
                    .find_map(|(_, _, e)| eff_find(e, slot, mode, kicked)),
                Effect::DealDamage { to, amount } => {
                    sel_find(to, slot).or_else(|| val_find(amount, slot))
                }
                // Each of slots 0..max_targets carries the divide filter, so
                // the cast/auto-target machinery collects "up to N targets".
                Effect::DealDamageDivided { filter, max_targets, .. } => {
                    if slot < *max_targets { Some(filter) } else { None }
                }
                Effect::SupportCounters { filter, max_targets } => {
                    if slot < *max_targets { Some(filter) } else { None }
                }
                Effect::PreventNextDamage { target, .. }
                | Effect::PreventNextDamageAndGainLife { target, .. }
                | Effect::PreventAllDamageThisTurn { target }
                | Effect::PreventAllCombatDamageInvolving { target } => sel_find(target, slot),
                Effect::Fight { attacker, defender } => {
                    sel_find(attacker, slot).or_else(|| sel_find(defender, slot))
                }
                Effect::ExchangeControl { a, b } => {
                    sel_find(a, slot).or_else(|| sel_find(b, slot))
                }
                Effect::GainLife { who, .. } | Effect::LoseLife { who, .. } => sel_find(who, slot),
                Effect::LoseHalfLife { who, .. }
                | Effect::MillHalf { who, .. }
                | Effect::DiscardHalf { who, .. }
                | Effect::SacrificeHalf { who, .. } => sel_find(who, slot),
                Effect::SetLifeTotal { who, .. } => sel_find(who, slot),
                Effect::Drain { from, to, .. } => sel_find(from, slot).or_else(|| sel_find(to, slot)),
                Effect::Draw { who, .. }
                | Effect::Mill { who, .. }
                | Effect::ExileTopOfLibrary { who, .. } => sel_find(who, slot),
                Effect::Discard { who, .. } => sel_find(who, slot),
                Effect::DiscardAnyNumber { who } => sel_find(who, slot),
                Effect::SetNoMaxHandSize { who } => sel_find(who, slot),
                Effect::SetMaxHandSize { who, .. } => sel_find(who, slot),
                Effect::Move { what, .. } => sel_find(what, slot),
                Effect::Destroy { what }
                | Effect::DestroyNoRegen { what }
                | Effect::ExileIfWouldDieThisTurn { what }
                | Effect::GrantFlashbackThisTurn { what }
                | Effect::GrantMiracle { what, .. }
                | Effect::Exile { what }
                | Effect::CounterSpell { what }
                | Effect::CounterSpellToZone { what, .. }
                | Effect::CounterAbility { what }
                | Effect::CounterUnlessPaid { what, .. }
                | Effect::CounterUnless { what, .. }
                | Effect::Suspect { what }
                | Effect::GainControl { what, .. } => sel_find(what, slot),
                Effect::UnlessPlayerPays { then, .. } => eff_find(then, slot, mode, kicked),
                Effect::PhaseOut { what } | Effect::Tap { what } | Effect::Untap { what, .. } => {
                    sel_find(what, slot).or_else(|| implicit_player_for_slot(what, slot))
                }
                Effect::PumpPT { what, .. }
                | Effect::SetBasePT { what, .. }
                | Effect::SwitchPT { what, .. }
                | Effect::DoublePower { what, .. } => {
                    sel_find(what, slot).or_else(|| implicit_creature_for_slot(what, slot))
                }
                Effect::BecomeCreature { what, .. } => sel_find(what, slot),
                Effect::GrantKeyword { what, .. }
                | Effect::GrantProtectionFromChosenColor { what, .. } => sel_find(what, slot),
                Effect::AddCounter { what, .. } | Effect::RemoveCounter { what, .. } => {
                    sel_find(what, slot)
                }
                Effect::RemoveAllCounters { what } => sel_find(what, slot),
                Effect::AddKeywordCounter { what, .. }
                | Effect::RemoveKeywordCounter { what, .. } => sel_find(what, slot),
                Effect::BecomeBasicLand { what, .. }
                | Effect::ResetCreature { what, .. } => sel_find(what, slot),
                Effect::Attach { what, to } => sel_find(what, slot).or_else(|| sel_find(to, slot)),
                Effect::CopySpell { what, .. }
                | Effect::CopySpellMayChooseTargets { what, .. }
                | Effect::CopySpellUnlessPaid { what, .. }
                | Effect::ChooseNewTargetsForSpell { what } => sel_find(what, slot),
                Effect::Sacrifice { who, .. } | Effect::SacrificeGreatestMV { who, .. } => {
                    sel_find(who, slot)
                }
                Effect::AddPoison { who, .. } => sel_find(who, slot),
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
            | Effect::SupportCounters { max_targets, .. } => Some(*max_targets),
            Effect::ChooseMode(modes) => match mode {
                Some(m) => modes.get(m).and_then(|e| e.distinct_target_count(None)),
                None => modes.iter().find_map(|e| e.distinct_target_count(None)),
            },
            Effect::MayDo { body, .. } | Effect::MayPay { body, .. } => {
                body.distinct_target_count(mode)
            }
            _ => None,
        }
    }
}
