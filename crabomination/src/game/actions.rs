use super::*;
use crate::card::{CardType, Keyword};
use crate::effect::{Effect, ManaPayload};
use crate::mana::{Color as ManaColor, ManaSymbol};

/// Per-pick snapshot of a permanent sacrificed to an additional cost:
/// `(id, power, is_creature, toughness, mana_value, is_artifact, is_vehicle,
/// colors)`. Stamped onto the resolution scratch so the spell body can read
/// `Value::Sacrificed*` and `Predicate::SacrificedWas*`.
type SacrificeSnapshot = (CardId, u32, bool, i32, u32, bool, bool, Vec<ManaColor>);

/// The grant sources live on the board right now, independent of which
/// permanent is asking — see [`GameState::grant_scan`]. Borrows the state it
/// was scanned from, so it can't outlive a board change.
#[derive(Default)]
pub(crate) struct GrantScan<'a> {
    /// Live `GrantActivatedAbility` statics past their CR 611.2 wrapper and
    /// `condition` gate: `(applies_to, ability, source)`.
    statics: Vec<(
        &'a crate::effect::Selector,
        &'a crate::effect::ActivatedAbility,
        &'a CardInstance,
    )>,
    /// Live `GrantActivatedAbilityFromGraveyard`: `(filter, ability, owning
    /// seat, source id)`.
    graveyard: Vec<(
        &'a crate::card::SelectionRequirement,
        &'a crate::effect::ActivatedAbility,
        usize,
        CardId,
    )>,
    /// Soulbond pairs whose bonus carries activated abilities and whose
    /// partner is still on the battlefield: `(source, partner, abilities)`.
    soulbond: Vec<(CardId, CardId, &'a [crate::effect::ActivatedAbility])>,
    /// Attached permanents carrying an `equipped_bonus`, matched per card by
    /// `attached_to`.
    equipment: Vec<&'a CardInstance>,
}

/// Skip-Ward check. Ward variants whose payment is trivially affordable
/// (free mana, 0 life, 0 discard) would always auto-pay and produce no
/// visible difference from no Ward at all — so we skip the stack-churn
/// of pushing the trigger. `SacrificeCreature` is never trivial since
/// the controller might have no creatures to sacrifice.
pub(crate) fn ward_cost_is_trivial(cost: &crate::card::WardCost) -> bool {
    use crate::card::WardCost;
    match cost {
        WardCost::Mana(c) => c.cmc() == 0,
        WardCost::Life(n) => *n == 0,
        WardCost::ManaAndLife(c, n) => c.cmc() == 0 && *n == 0,
        WardCost::Discard(n) | WardCost::DiscardRandom(n) => *n == 0,
        WardCost::DiscardMatching(_, n) => *n == 0,
        // Discarding your hand is never trivial as a Perplex-style counter cost,
        // but as a Ward tax it's free when the hand is already empty.
        WardCost::DiscardHand => false,
        WardCost::Blight(n) => *n == 0,
        WardCost::CollectEvidence(n) => *n == 0,
        WardCost::ExileFromGraveyard(n)
        | WardCost::BottomFromGraveyard(n)
        | WardCost::DamageFromSource(n) => *n == 0,
        WardCost::SacrificeCreature
        | WardCost::SacrificeMatching(_)
        | WardCost::ReturnMatchingToHand(..)
        | WardCost::ExileTopFromGraveyardMatching(_)
        | WardCost::ReturnMatchingFromGraveyardToHand(_) => false,
        WardCost::SacrificeMatchingN(_, n) => *n == 0,
        // "{X}" is only free when the declared X was 0, which the caller
        // can't see here.
        WardCost::GenericXFromCost | WardCost::GenericCountersOnSource(_) => false,
        WardCost::SacrificePermanents(n) => *n == 0,
        // Dynamic — the source's power can change before payment.
        WardCost::GenericSourcePower | WardCost::LifeSourcePower => false,
        WardCost::RemoveCounterFromPermanent
        | WardCost::ManaCostOfAttached
        | WardCost::ManaOrLife(_, _)
        | WardCost::SacrificeAttachedHost => false,
    }
}

impl GameState {
    /// The alternative cost `p` may use to cast the hand card `card_id`: the
    /// printed one, or the CR 118.9 WUBRG cost Fist of Suns grants to every
    /// spell its controller casts.
    pub(crate) fn effective_alternative_cost(
        &self,
        p: usize,
        card_id: CardId,
    ) -> Option<crate::card::AlternativeCost> {
        let printed =
            self.players[p].hand.iter().find(|c| c.id == card_id)?.definition.alternative_cost.clone();
        if printed.is_some() {
            return printed;
        }
        let five_color = self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| {
                        matches!(sa.effect, crate::effect::StaticEffect::FiveColorAlternativeCost)
                    })
        });
        if five_color {
            return Some(crate::card::AlternativeCost {
                mana_cost: crate::mana::cost(&[
                    crate::mana::w(),
                    crate::mana::u(),
                    crate::mana::b(),
                    crate::mana::r(),
                    crate::mana::g(),
                ]),
                ..Default::default()
            });
        }
        // Kentaro, the Smiling Cat — "pay {X} rather than the mana cost for
        // [filter] spells you cast, where X is that spell's mana value."
        let card = self.players[p].hand.iter().find(|c| c.id == card_id)?;
        let generic_alt = self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition.static_abilities.iter().any(|sa| match &sa.effect {
                    crate::effect::StaticEffect::GenericAlternativeCostForFilter { filter } => {
                        self.evaluate_requirement_on_card(filter, card, p)
                    }
                    _ => false,
                })
        });
        if generic_alt {
            return Some(crate::card::AlternativeCost {
                mana_cost: crate::mana::cost(&[crate::mana::generic(card.definition.cost.cmc())]),
                ..Default::default()
            });
        }
        // Dream Halls — every seat may discard a card sharing a colour with
        // the spell instead of paying for it. Colourless spells share no
        // colour, so they get no discount.
        let dream_halls = self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(
                    sa.effect,
                    crate::effect::StaticEffect::DiscardColorSharingCardAlternativeCost
                )
            })
        });
        if dream_halls {
            let colors = card.definition.printed_colors();
            let filter = colors
                .into_iter()
                .map(crate::card::SelectionRequirement::HasColor)
                .reduce(|a, b| a.or(b))?;
            return Some(crate::card::AlternativeCost {
                discard_filters: vec![(filter, 1)],
                ..Default::default()
            });
        }
        None
    }
}

/// The "remove N counters from among permanents you control" cost of an
/// activated ability, normalized across the any-kind
/// (`remove_counter_among_filter`) and kind-restricted
/// (`remove_counter_among_kinds`) spellings. `None` kinds = any kind.
fn counter_drain_cost(
    ability: &crate::effect::ActivatedAbility,
) -> Option<(Option<Vec<crate::card::CounterType>>, u32, crate::card::SelectionRequirement)> {
    if let Some((kind, count, filter)) = ability.remove_counter_among_filter.as_ref() {
        return Some((kind.map(|k| vec![k]), *count, filter.clone()));
    }
    let (kinds, count, filter) = ability.remove_counter_among_kinds.as_ref()?;
    Some((Some(kinds.clone()), *count, filter.clone()))
}

/// Counters on `c` that a `counter_drain_cost` of the given kinds could take.
fn drainable_counters(c: &CardInstance, kinds: Option<&[crate::card::CounterType]>) -> u32 {
    match kinds {
        Some(ks) => ks.iter().map(|k| c.counter_count(*k)).sum(),
        None => c.counters.values().sum(),
    }
}

/// Returns true if the given effect is purely a mana ability — only adds
/// mana and uses no targets. Mana abilities resolve immediately without the stack.
/// The three modifications that write `ComputedPermanent.subtypes.land_types`
/// (Spreading Seas, Blood Moon, Urborg, Magical Hack). With none of them in
/// scope a permanent's computed land types are its printed ones, which is
/// what lets `effective_mana_abilities_with` skip the per-card layer pass.
/// Keep in step with `layers::compute_permanent_pass` — a fourth land-type
/// writer added there must be added here or lands stop losing their mana
/// abilities.
fn rewrites_land_types(e: &crate::game::layers::ContinuousEffect) -> bool {
    use crate::game::layers::Modification as M;
    matches!(
        e.modification,
        M::AddLandType(_) | M::SetLandTypes(_) | M::ReplaceBasicLandType(..)
    )
}

/// Public wrapper for `is_mana_ability` — read by `SelectionRequirement::
/// HasNonManaActivatedAbility` (Tsabo's Web).
pub fn is_mana_ability_public(effect: &Effect) -> bool {
    is_mana_ability(effect)
}

pub(crate) fn is_mana_ability(effect: &Effect) -> bool {
    // CR 605.1a — a mana ability could add mana, isn't a loyalty ability,
    // doesn't target, and doesn't have an illegal trigger. It may still carry
    // incidental non-stack riders (Altar of the Pantheon's "gain 1 life").
    fn produces_mana(e: &Effect) -> bool {
        match e {
            Effect::AddMana { .. } => true,
            Effect::Seq(steps) => steps.iter().any(produces_mana),
            Effect::If { then, else_, .. } => produces_mana(then) || produces_mana(else_),
            _ => false,
        }
    }
    fn mana_compatible(e: &Effect) -> bool {
        match e {
            Effect::AddMana { .. } | Effect::Noop => true,
            // Incidental you-only life gain (Altar of the Pantheon).
            Effect::GainLife { who: crate::effect::Selector::You, .. } => true,
            // Incidental non-targeting self-counter (Twitching Doll's "put a
            // nest counter on this creature") — CR 605.1a rider, no stack use.
            Effect::AddCounter { what: crate::effect::Selector::This, .. } => true,
            // "This land doesn't untap during your next untap step" (the CHK
            // slow duals) — another non-targeting CR 605.1a rider.
            Effect::SkipNextUntap { what: crate::effect::Selector::This } => true,
            // CR 605.1a/603.7 — a reflexive "when you do" rider triggers OFF
            // the mana ability; it goes on the stack itself but doesn't stop
            // the ability being a mana ability (Rubble Rouser's "Add {R}.
            // When you do, deal 1 to each opponent").
            Effect::ReflexiveTrigger { .. } => true,
            Effect::Seq(steps) => steps.iter().all(mana_compatible),
            // A board-state-conditional that only ever adds mana on both
            // branches is still a mana ability (Ilysian Caryatid's "add one
            // of any color; add two instead if you control a power-4+
            // creature").
            Effect::If { then, else_, .. } => mana_compatible(then) && mana_compatible(else_),
            _ => false,
        }
    }
    produces_mana(effect) && mana_compatible(effect)
}

/// The set of colours `card`'s untapped mana abilities can produce, in
/// WUBRG order (a Forest → `[Green]`, a dual → two, Birds → all five, a
/// colorless rock → `[]`). Used as a source's "signature" so the
/// manual-tap decision can tell interchangeable sources (same signature)
/// from genuinely different ones.
fn source_color_signature(card: &crate::card::CardInstance) -> Vec<ManaColor> {
    ManaColor::ALL
        .into_iter()
        .filter(|c| {
            card.definition
                .activated_abilities
                .iter()
                .any(|a| is_mana_ability(&a.effect) && effect_produces_color(&a.effect, *c))
        })
        .collect()
}

/// Broad permanent kind for the manual-tap signature: tapping a Forest vs
/// a Mox Emerald matters even though both make `[Green]` (a Wasteland /
/// Shatter cares which stays untapped), so same-color sources of
/// *different kinds* count as a genuine tapping choice.
fn source_kind(card: &crate::card::CardInstance) -> u8 {
    if card.definition.is_land() {
        0
    } else if card.definition.is_creature() {
        // Creature first: a mana dork riding an artifact body (Solemn-style)
        // taps like a creature (summoning sickness, combat).
        2
    } else if card.definition.is_artifact() {
        1
    } else {
        3
    }
}

/// WUBRG index, matching [`crate::mana::Color::ALL`] order.
pub fn color_index(c: ManaColor) -> usize {
    match c {
        ManaColor::White => 0,
        ManaColor::Blue => 1,
        ManaColor::Black => 2,
        ManaColor::Red => 3,
        ManaColor::Green => 4,
    }
}

/// A cached untapped mana source: what it can make and what it costs to
/// activate. See `GameState::mana_source_table`.
struct ManaSourceInfo {
    id: CardId,
    /// Ability index used when any mana will do (the generic portion).
    first_idx: usize,
    rank: u8,
    /// Producible colours, with the ability index that makes each in
    /// `color_idx[color_index(c)]` (only read where `colors` contains the
    /// colour). A bitmask plus a fixed array rather than a
    /// `Vec<(ManaColor, usize)>`: at most five entries, and the `Vec` was
    /// one heap allocation per untapped source per `auto_tap_for_cost`.
    colors: crate::mana::ColorSet,
    color_idx: [usize; 5],
}

impl ManaSourceInfo {
    /// How replaceable this source is among `others`: for each colour it
    /// makes, how many *other* listed sources also make that colour,
    /// minimised over its colours. A colourless source has nothing to
    /// preserve and scores `u32::MAX`.
    ///
    /// This is what stops the generic portion of a cost eating a splash.
    /// Paying `{2}{B}` off 8 Swamp / 6 Forest / 3 Island, the engine
    /// reserved a Swamp for the `{B}` and paid the `{2}` in battlefield
    /// order — as likely to be an Island as anything else, stranding the
    /// blue cards the three Islands exist to cast. Spending the most
    /// replaceable source first takes a Swamp (7 backups) before a Forest
    /// (5) before an Island (2), and never changes *whether* the current
    /// cost can be paid — only which of several interchangeable sources
    /// pays it.
    fn redundancy(&self, others: &[ManaSourceInfo]) -> u32 {
        if self.colors.is_empty() {
            return u32::MAX;
        }
        self.colors
            .iter()
            .map(|c| {
                others.iter().filter(|o| o.id != self.id && o.colors.contains(c)).count() as u32
            })
            .min()
            .unwrap_or(u32::MAX)
    }
}

/// Kind-aware source signature — see [`source_kind`] /
/// [`source_color_signature`].
fn source_kind_signature(card: &crate::card::CardInstance) -> (u8, Vec<ManaColor>) {
    (source_kind(card), source_color_signature(card))
}

/// Pull the "when you cast this spell" (`EventKind::SpellCast` +
/// `EventScope::SelfSource`) triggers off a card. Used by the cast paths
/// to push these onto the stack above the cast spell so they resolve
/// before the spell itself. Returns the trigger's optional filter
/// predicate alongside its effect so the caller can gate the trigger
/// fire on the predicate (e.g. Infusion's LifeGainedThisTurnAtLeast).
fn collect_self_cast_triggers(
    card: &crate::card::CardInstance,
) -> Vec<(Option<crate::card::Predicate>, Effect)> {
    use crate::effect::{EventKind, EventScope};
    card.definition
        .triggered_abilities
        .iter()
        .filter(|t| {
            t.event.kind == EventKind::SpellCast
                && matches!(t.event.scope, EventScope::SelfSource)
        })
        .map(|t| (t.event.filter.clone(), t.effect.clone()))
        .collect()
}

/// Count distinct colors of mana that decreased between two pool
/// snapshots — i.e. the spell's converge value.
/// Colored "or pay" halves of additional cast costs whose resource half
/// isn't available right now (`ExileFromGraveyardOrPay` with a thin
/// graveyard): the printed pay cost joins the spell's cost
/// symbol-for-symbol — colored pips included, which the generic
/// [`extra_cost_for_spell`] tax channel can't express.
pub(crate) fn or_pay_cost_symbols(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
) -> Vec<crate::mana::ManaSymbol> {
    let mut out = Vec::new();
    for c in &card.definition.additional_cast_cost {
        if let crate::card::AdditionalCastCost::ExileFromGraveyardOrPay { filter, count, pay } = c
        {
            let matches = state.players[caster]
                .graveyard
                .iter()
                .filter(|c| state.evaluate_requirement_on_card(filter, c, caster))
                .count() as u32;
            if matches < *count {
                out.extend(pay.symbols.iter().cloned());
            }
        }
    }
    out
}

fn converge_count(before: &crate::mana::ManaPool, after: &crate::mana::ManaPool) -> u32 {
    use crate::mana::Color;
    let mut count = 0u32;
    for color in Color::ALL {
        if before.amount(color) > after.amount(color) {
            count += 1;
        }
    }
    count
}

/// Per-color breakdown of the mana drained between two pool snapshots
/// (CR 601 — stamped onto `CardInstance.cast_mana_spent_by_color` for
/// Adamant / Void Mirror reads). Colorless is not a color and is omitted.
fn spent_by_color(
    before: &crate::mana::ManaPool,
    after: &crate::mana::ManaPool,
) -> Vec<(crate::mana::Color, u32)> {
    use crate::mana::Color;
    Color::ALL
        .iter()
        .filter_map(|&c| {
            let spent = before.amount(c).saturating_sub(after.amount(c));
            (spent > 0).then_some((c, spent))
        })
        .collect()
}

/// CR 701.67 — resolve a card's printed waterbend amount given the chosen X.
/// Returns `(amount, optional)`, or `None` if the card has no waterbend.
/// Supports `Value::Const(n)` and the chosen-X form (`Value::XFromCost`);
/// other expressions resolve to 0.
pub(crate) fn waterbend_amount(
    def: &crate::card::CardDefinition,
    x_value: Option<u32>,
) -> Option<(u32, bool)> {
    use crate::effect::Value;
    def.waterbend.as_ref().map(|wb| {
        let amt = match wb.amount {
            Value::Const(n) => n.max(0) as u32,
            Value::XFromCost => x_value.unwrap_or(0),
            _ => 0,
        };
        (amt, wb.optional)
    })
}

/// Walk the battlefield's static abilities + per-player tax charges to
/// compute the total extra generic mana the caster owes for casting `card`.
///
/// Honors:
///   * `StaticEffect::AdditionalCostAfterFirstSpell` (Damping Sphere): if
///     the caster has already cast at least one spell this turn and the
///     spell matches the static's `filter`, charge `amount` more.
///   * `Player.first_spell_tax_charges` (Chancellor of the Annex): each
///     pending charge taxes the caster's *next* spell {1} more. Consumed by
///     the caller on a successful cast (we only **read** here so callers
///     can see the tax before payment; the caster path decrements after).
/// CR 702.34a — the card's flashback-only additional cost(s), with the cast's
/// chosen `x` folded into any X-dependent count (Conflagrate's "Discard X
/// cards"). `cast_flashback` validates + pays these on top of the flashback
/// mana cost.
pub(crate) fn flashback_additional_costs(
    def: &crate::card::CardDefinition,
    x: u32,
) -> Vec<crate::card::AdditionalCastCost> {
    use crate::card::AdditionalCastCost as A;
    def.flashback_additional_cost
        .iter()
        .map(|c| match c {
            A::DiscardXFromCost => A::Discard { count: x, filter: None },
            A::DiscardXRandomFromCost => A::DiscardRandom { count: x },
            A::ExileFromGraveyardXFromCost { filter } => {
                A::ExileFromGraveyard { filter: filter.clone(), count: x }
            }
            other => other.clone(),
        })
        .collect()
}

/// CR 702.122 Strive (and Fireball's generic sibling): "this spell costs
/// [cost] more to cast for each target beyond the first". Returns the total
/// surcharge for `extra_targets` filled additional slots.
pub fn strive_cost_for_spell(
    card: &crate::card::CardInstance,
    extra_targets: usize,
) -> crate::mana::ManaCost {
    let mut out = crate::mana::ManaCost::default();
    if let Some(per) = &card.definition.cost_per_extra_target {
        for _ in 0..extra_targets {
            out.symbols.extend(per.symbols.iter().cloned());
        }
    }
    out
}

pub fn extra_cost_for_spell(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
    target: Option<&crate::game::Target>,
) -> u32 {
    use crate::effect::StaticEffect;
    let mut tax = 0u32;
    if state.players[caster].first_spell_tax_charges > 0 {
        tax += 1;
    }
    // "Costs {N} more if it targets a [filter]" (Vanish into Eternity).
    if let Some((filter, n)) = &card.definition.cost_increase_if_targets
        && let Some(t) = target
        && state.evaluate_requirement_static(filter, t, caster, None)
    {
        tax += n;
    }
    // "Reveal a [filter] card from your hand or pay {N}" (Silvergill Adept):
    // no matching hand card → the pay half joins the cost. Same shape for
    // "sacrifice a [filter] or pay {N}" (Bayou Groff) against the battlefield.
    for ac in &card.definition.additional_cast_cost {
        match ac {
            crate::card::AdditionalCastCost::RevealFromHandOrPay { filter, pay } => {
                let has_match = state.players[caster].hand.iter().any(|c| {
                    c.id != card.id && state.evaluate_requirement_on_card(filter, c, caster)
                });
                if !has_match {
                    tax += pay;
                }
            }
            crate::card::AdditionalCastCost::SacrificeOrPay { filter, pay } => {
                let has_match = state.battlefield.iter().any(|c| {
                    c.controller == caster
                        && state.evaluate_requirement_on_card(filter, c, caster)
                });
                if !has_match {
                    tax += pay;
                }
            }
            // No forage material → the pay half joins the cost.
            crate::card::AdditionalCastCost::ForageOrPay { pay }
                if !state.can_forage(caster) =>
            {
                tax += pay;
            }
            _ => {}
        }
    }
    let already_cast = state.players[caster].spells_cast_this_turn;
    for src in &state.battlefield {
        for sa in &src.definition.static_abilities {
            match &sa.effect {
                StaticEffect::AdditionalCostAfterFirstSpell { filter, amount }
                    if already_cast > 0
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    tax += amount;
                }
                StaticEffect::AdditionalCost { filter, amount }
                    if state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    tax += amount;
                }
                // Grand Arbiter Augustin IV: opponents' spells cost more (the
                // controller's own spells are exempt).
                StaticEffect::OpponentSpellsCostMore { filter, amount }
                    if src.controller != caster
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    tax += amount;
                }
                StaticEffect::NamedSpellTax { amount }
                    if src.named_card.as_deref() == Some(card.definition.name) =>
                {
                    tax += amount;
                }
                // Tithe Taker: on its controller's turn, opponents' spells cost
                // {amount} more (the controller is exempt).
                StaticEffect::OpponentActivityCostsMoreOnYourTurn { amount }
                    if src.controller != caster
                        && src.controller == state.active_player_idx =>
                {
                    tax += amount;
                }
                // Defense Grid: each spell costs {amount} more except during its
                // controller's turn (CR — read off the caster's active status).
                StaticEffect::SpellsCostMoreExceptOnControllerTurn { amount }
                    if caster != state.active_player_idx =>
                {
                    tax += amount;
                }
                // Jubilant Skybonder: opponents' spells targeting a qualifying
                // permanent the source controls cost {amount} more.
                StaticEffect::TaxOpponentSpellsTargeting { target_filter, amount }
                    if src.controller != caster =>
                {
                    if let Some(crate::game::Target::Permanent(pid)) = target
                        && let Some(tc) = state.battlefield_find(*pid)
                        && tc.controller == src.controller
                        && state.evaluate_requirement_on_card(target_filter, tc, src.controller)
                    {
                        tax += amount;
                    }
                }
                // Hum of the Radix: a matching spell costs {1} more for each
                // matching permanent its OWN controller controls.
                StaticEffect::SpellTaxPerControllerPermanent { spell_filter, count_filter }
                    if state.evaluate_requirement_on_card(spell_filter, card, caster) =>
                {
                    tax += state
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == caster
                                && state.evaluate_requirement_on_card(count_filter, c, caster)
                        })
                        .count() as u32;
                }
                // Sphinx of New Prahv: opponents' spells targeting the Sphinx
                // itself cost {amount} more.
                StaticEffect::TaxOpponentSpellsTargetingThis { amount }
                    if src.controller != caster =>
                {
                    if let Some(crate::game::Target::Permanent(pid)) = target
                        && *pid == src.id
                    {
                        tax += amount;
                    }
                }
                _ => {}
            }
        }
    }
    // Turn-scoped taxes (Elspeth Conquers Death II): opponents of the
    // entry's controller pay until that player's next turn.
    for t in &state.turn_scoped_spell_taxes {
        if t.controller != caster && state.evaluate_requirement_on_card(&t.filter, card, caster) {
            tax += t.amount;
        }
    }

    tax
}

/// Consume one Chancellor-of-the-Annex tax charge from `caster`, if any.
/// Called by every cast path immediately after the spell successfully
/// resolves payment, so each first-spell-tax charge is single-use.
pub(crate) fn consume_first_spell_tax(state: &mut crate::game::GameState, caster: usize) {
    if state.players[caster].first_spell_tax_charges > 0 {
        state.players[caster].first_spell_tax_charges -= 1;
    }
}

/// Sum all generic-mana cost reductions applicable to a spell being cast.
///
/// Supports two flavors:
///   * `StaticEffect::CostReduction { filter, amount }` — flat per-spell
///     reduction whose `filter` matches the cast card.
///   * `StaticEffect::CostReductionTargetingFilter { spell_filter,
///     target_filter, amount }` — Killian-style "if the spell targets a
///     creature, it costs {2} less". Honors the cast's chosen target via
///     `target` (so Lightning Bolt at face counts as targeting a player,
///     Lightning Bolt at a creature counts as targeting a creature).
///
/// CR 601.2f / 117.7c: cost reductions can never reduce a colored or X
/// pip. The caller funnels the returned reduction through
/// `ManaCost::reduce_generic`, which clamps at the generic pip total.
/// True if the filter tree names a controller clause explicitly
/// (`ControlledByYou` / `ControlledByOpponent`) — such filters replace the
/// implicit "you control" scope of affinity-style counts.
fn requirement_mentions_controller(req: &crate::card::SelectionRequirement) -> bool {
    use crate::card::SelectionRequirement as R;
    match req {
        R::ControlledByYou | R::ControlledByOpponent => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_controller(a) || requirement_mentions_controller(b)
        }
        R::Not(inner) => requirement_mentions_controller(inner),
        _ => false,
    }
}

/// CR 601.2f — the COLORED half of static cost reduction
/// (`StaticEffect::ColoredCostReduction`, Ragemonger). Returns the summed
/// reduction cost; callers fold it in with `ManaCost::reduce_by_cost` right
/// after the generic reduction.
pub fn colored_cost_reduction_for_spell(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
) -> crate::mana::ManaCost {
    use crate::effect::StaticEffect;
    let mut out = crate::mana::ManaCost::default();
    for src in &state.battlefield {
        if src.controller != caster {
            continue;
        }
        for sa in &src.definition.static_abilities {
            if let StaticEffect::ColoredCostReduction { filter, less } = &sa.effect
                && state.evaluate_requirement_on_card(filter, card, caster)
            {
                out.symbols.extend(less.symbols.iter().cloned());
            }
        }
    }
    out
}

/// CR 601.2f — the COLORED half of static cost *increases*
/// (`StaticEffect::ColoredSpellTax`, the Invasion Leech cycle). Only the
/// source's controller pays it. Callers append the returned pips to the cost
/// before any reduction runs, so a discount can never eat the surcharge.
pub fn colored_spell_tax_for_spell(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
) -> crate::mana::ManaCost {
    use crate::effect::StaticEffect;
    let mut out = crate::mana::ManaCost::default();
    for src in &state.battlefield {
        if src.controller != caster {
            continue;
        }
        for sa in &src.definition.static_abilities {
            if let StaticEffect::ColoredSpellTax { filter, more } = &sa.effect
                && state.evaluate_requirement_on_card(filter, card, caster)
            {
                out.symbols.extend(more.symbols.iter().cloned());
            }
        }
    }
    out
}

pub fn cost_reduction_for_spell(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
    target: Option<&crate::game::Target>,
) -> u32 {
    cost_reduction_for_spell_zoned(state, caster, card, target, false)
}

/// Like `cost_reduction_for_spell`, but `from_graveyard` toggles the
/// graveyard-cast-only statics (Gravebreaker Lamia). The graveyard-cast paths
/// (flashback / retrace / escape / disturb / aftermath) pass `true`.
impl crate::game::GameState {
    /// Catalyst Stone (CR 702.34) — `(less, more)` generic shift applied to
    /// `p`'s flashback costs by battlefield statics.
    pub(crate) fn flashback_cost_shift(&self, p: usize) -> (u32, u32) {
        use crate::effect::StaticEffect;
        let (mut less, mut more) = (0u32, 0u32);
        for c in &self.battlefield {
            for sa in &c.definition.static_abilities {
                match self.active_static(&sa.effect, c) {
                    Some(StaticEffect::FlashbackCostReduction { amount })
                        if c.controller == p =>
                    {
                        less += amount
                    }
                    Some(StaticEffect::OpponentFlashbackTax { amount })
                        if !self.same_team(c.controller, p) =>
                    {
                        more += amount
                    }
                    _ => {}
                }
            }
        }
        (less, more)
    }
}

pub fn cost_reduction_for_spell_zoned(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
    target: Option<&crate::game::Target>,
    from_graveyard: bool,
) -> u32 {
    cost_reduction_for_spell_full(state, caster, card, target, from_graveyard, false)
}

/// Full cost-reduction scan with both `from_graveyard` and `from_exile` zone
/// toggles. Exile-cast paths (foretell / adventure-creature / plotted / impulse
/// pay-own-cost) pass `from_exile: true` so `ExileCastCostReduction` (Doc
/// Aurlock) and every zone-agnostic reduction apply.
pub fn cost_reduction_for_spell_full(
    state: &crate::game::GameState,
    caster: usize,
    card: &crate::card::CardInstance,
    target: Option<&crate::game::Target>,
    from_graveyard: bool,
    from_exile: bool,
) -> u32 {
    use crate::effect::StaticEffect;
    let mut reduction = 0u32;
    // CR 315.5 — a face-up conspiracy's cost statics apply from the command
    // zone (Hymn of the Wilds, Brago's Favor), so walk those too.
    for src in state.all_static_sources() {
        for sa in &src.definition.static_abilities {
            // CR 716.2 — a Class's higher-level statics only apply once the
            // permanent has reached that level (Artist's Talent level 2).
            let effect = match &sa.effect {
                StaticEffect::WhileClassLevelAtLeast { n, inner }
                    if src.class_level >= *n =>
                {
                    inner.as_ref()
                }
                StaticEffect::WhileClassLevelAtLeast { .. } => continue,
                other => other,
            };
            match effect {
                StaticEffect::GraveyardCastCostReduction { amount }
                    if from_graveyard && src.controller == caster =>
                {
                    reduction += amount;
                }
                StaticEffect::ExileCastCostReduction { amount }
                    if from_exile && src.controller == caster =>
                {
                    reduction += amount;
                }
                StaticEffect::CostReduction { filter, amount }
                    if src.controller == caster
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    reduction += amount;
                }
                // Helm of Awakening — table-wide, so no controller gate.
                StaticEffect::AllPlayersSpellsCostLess { amount } => {
                    reduction += amount;
                }
                // "The first [filter] spell you cast each turn costs {N} less"
                // — spent as soon as one matching spell has been cast.
                StaticEffect::FirstMatchingSpellEachTurnCostsLess { filter, amount }
                    if src.controller == caster
                        && state.evaluate_requirement_on_card(filter, card, caster)
                        && !state.players[caster].spell_ids_cast_this_turn.iter().any(|id| {
                            state.find_card_anywhere(*id).is_some_and(|c| {
                                state.evaluate_requirement_on_card(filter, c, caster)
                            })
                        }) =>
                {
                    reduction += amount;
                }
                // Battlefield Thaumaturge — {N} less per creature the
                // instant/sorcery targets.
                StaticEffect::YourISSpellsCostLessPerTargetCreature { amount }
                    if src.controller == caster
                        && (card.definition.is_instant() || card.definition.is_sorcery()) =>
                {
                    let targets_creature = target.is_some_and(|t| {
                        matches!(t, crate::game::Target::Permanent(id)
                            if state.battlefield_find(*id).is_some_and(|c| c.definition.is_creature()))
                    });
                    if targets_creature {
                        reduction += amount;
                    }
                }
                StaticEffect::NamedSpellCostReduction { amount }
                    if src.controller == caster
                        && src.named_card.as_deref() == Some(card.definition.name) =>
                {
                    reduction += amount;
                }
                // Mistform Warchief — the shared type is read off the source's
                // *computed* types, so its own {T} type-change counts.
                StaticEffect::SharedCreatureTypeSpellCostReduction { amount }
                    if src.controller == caster && card.definition.is_creature() =>
                {
                    let mine = state
                        .computed_permanent(src.id)
                        .map(|cp| cp.subtypes.creature_types.clone())
                        .unwrap_or_else(|| src.definition.subtypes.creature_types.clone());
                    if card
                        .definition
                        .subtypes
                        .creature_types
                        .iter()
                        .any(|t| mine.contains(t))
                    {
                        reduction += amount;
                    }
                }
                StaticEffect::CostReductionPerControllerExperience { filter }
                    if src.controller == caster
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    reduction += state.players[caster].experience;
                }
                StaticEffect::CostReductionByValue { filter, amount }
                    if src.controller == caster
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    let ctx = crate::game::effects::EffectContext::for_trigger(
                        src.id, caster, None, 0,
                    );
                    reduction += state.evaluate_value(amount, &ctx).max(0) as u32;
                }
                StaticEffect::CostReductionPerCounterOnSource { filter, kind }
                    if src.controller == caster
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    reduction += src.counter_count(*kind);
                }
                StaticEffect::CostReductionBySourcePower { filter }
                    if src.controller == caster
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    reduction +=
                        state.computed_permanent(src.id).map(|c| c.power.max(0)).unwrap_or(0) as u32;
                }
                StaticEffect::CostReductionWhile { filter, amount, condition }
                    if src.controller == caster
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    let ctx = crate::game::effects::EffectContext {
                        source: Some(src.id),
                        ..crate::game::effects::EffectContext::for_spell(caster, None, 0, 0)
                    };
                    if state.evaluate_predicate(condition, &ctx) {
                        reduction += amount;
                    }
                }
                StaticEffect::CostReductionDuringOpponentsTurn { filter, amount }
                    if src.controller == caster
                        && state.active_player_idx != caster
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    reduction += amount;
                }
                StaticEffect::CostReductionNthSpell { filter, nth, amount }
                    if src.controller == caster
                        && state.players[caster].spells_cast_this_turn + 1 == *nth
                        && state.evaluate_requirement_on_card(filter, card, caster) =>
                {
                    reduction += amount;
                }
                StaticEffect::CostReductionFirstCreatureSpell { amount }
                    if src.controller == caster
                        && state.players[caster].creatures_cast_this_turn == 0
                        && card.definition.card_types.contains(&CardType::Creature) =>
                {
                    reduction += amount;
                }
                StaticEffect::CostReductionFirstInstantOrSorcery { amount }
                    if src.controller == caster
                        && state.players[caster].instants_or_sorceries_cast_this_turn == 0
                        && (card.definition.is_instant() || card.definition.is_sorcery()) =>
                {
                    reduction += amount;
                }
                StaticEffect::CostReductionFirstInstantOrSorceryPerValue { per }
                    if src.controller == caster
                        && state.players[caster].instants_or_sorceries_cast_this_turn == 0
                        && (card.definition.is_instant() || card.definition.is_sorcery()) =>
                {
                    let ctx = crate::game::effects::EffectContext::for_ability(
                        src.id,
                        src.controller,
                        None,
                    );
                    reduction += state.evaluate_value(per, &ctx).max(0) as u32;
                }
                StaticEffect::CostReductionTargetingFilter {
                    spell_filter,
                    target_filter,
                    amount,
                } => {
                    if src.controller != caster {
                        continue;
                    }
                    if !state.evaluate_requirement_on_card(spell_filter, card, caster) {
                        continue;
                    }
                    let Some(tgt) = target else { continue };
                    if state.evaluate_requirement_static(target_filter, tgt, caster, Some(card.id)) {
                        reduction += amount;
                    }
                }
                StaticEffect::ChosenTypeSpellCostReduction { amount }
                    if src.controller == caster
                        && card.definition.is_creature()
                        && src.chosen_creature_type.is_some_and(|ct| {
                            card.has_keyword(&crate::card::Keyword::Changeling)
                                || card.definition.subtypes.creature_types.contains(&ct)
                        }) =>
                {
                    reduction += amount;
                }
                StaticEffect::GrantAffinityToISSpells { permanent_filter } => {
                    // "Instant and sorcery spells you cast have Affinity for
                    // [permanent_filter]" — only fires on the controller's
                    // IS spells. Counts every battlefield permanent matching
                    // `permanent_filter` and reduces by 1 per match.
                    if src.controller != caster {
                        continue;
                    }
                    if !card.definition.is_instant() && !card.definition.is_sorcery() {
                        continue;
                    }
                    let count = state
                        .battlefield
                        .iter()
                        .filter(|c| state.evaluate_requirement_on_card(permanent_filter, c, caster))
                        .count();
                    reduction = reduction.saturating_add(count as u32);
                }
                StaticEffect::GrantAffinityToSpells { spell_filter, permanent_filter } => {
                    // "[spell_filter] spells you cast have Affinity for
                    // [permanent_filter]" (Tezzeret, Master of the Bridge).
                    if src.controller != caster {
                        continue;
                    }
                    if !state.evaluate_requirement_on_card(spell_filter, card, caster) {
                        continue;
                    }
                    let count = state
                        .battlefield
                        .iter()
                        .filter(|c| state.evaluate_requirement_on_card(permanent_filter, c, caster))
                        .count();
                    reduction = reduction.saturating_add(count as u32);
                }
                _ => {}
            }
        }
    }
    // Card-intrinsic Affinity-for-[filter] cost reduction: "{1} less for
    // each [filter]" baked onto the spell card itself. Counts every
    // battlefield permanent matching `affinity_filter`. CR 601.2f / 117.7c —
    // generic-only, the colored-pip clamp happens in
    // `ManaCost::reduce_generic` once the caller folds this back into the
    // cost.
    if let Some(filter) = &card.definition.affinity_filter {
        let count = state
            .battlefield
            .iter()
            .filter(|c| state.evaluate_requirement_on_card(filter, c, caster))
            .count();
        reduction = reduction.saturating_add(count as u32);
    }
    // Card-intrinsic "Affinity-for-cards-in-your-graveyard" cost reduction:
    // "{1} less for each [filter] card in your graveyard" (The Dawning
    // Archaic, Tolarian Terror). Generic-only, clamped by the caller.
    if let Some(filter) = &card.definition.affinity_graveyard_filter {
        let count = state.players[caster]
            .graveyard
            .iter()
            .filter(|c| state.evaluate_requirement_on_card(filter, c, caster))
            .count();
        reduction = reduction.saturating_add(count as u32);
    }
    // Card-intrinsic target-conditional reduction (Ride's End): "{amount}
    // less if it targets a permanent matching `filter`." Generic-only.
    if let Some((filter, amount)) = &card.definition.self_cost_reduction_if_target
        && let Some(tgt) = target
        && state.evaluate_requirement_static(filter, tgt, caster, Some(card.id))
    {
        reduction = reduction.saturating_add(*amount);
    }
    // Card-intrinsic board-state-gated flat reductions (Pearl of Wisdom,
    // Geistlight Snare): each "{amount} less if you control a permanent
    // matching `filter`" clause applies independently.
    for (filter, amount) in &card.definition.self_cost_reduction_if_control {
        if state
            .battlefield
            .iter()
            .any(|c| state.evaluate_requirement_on_card(filter, c, caster))
        {
            reduction = reduction.saturating_add(*amount);
        }
    }
    // Card-intrinsic "costs {amount} less if it's night" (Moonrager's Slash).
    if let Some(amount) = card.definition.self_cost_reduction_if_night
        && state.day_night == Some(crate::game::types::DayNight::Night)
    {
        reduction = reduction.saturating_add(amount);
    }
    // Card-intrinsic Delirium reduction — "{amount} less while four or more
    // card types are in your graveyard" (Drag to the Roots).
    if let Some(amount) = card.definition.self_cost_reduction_if_delirium
        && state.delirium_active(caster)
    {
        reduction = reduction.saturating_add(amount);
    }
    // Card-intrinsic crime reduction — "{amount} less if you've committed a
    // crime this turn" (Seize the Secrets, CR 700.13).
    if let Some(amount) = card.definition.self_cost_reduction_if_crime
        && state.players[caster].committed_crime_this_turn
    {
        reduction = reduction.saturating_add(amount);
    }
    // Card-intrinsic "costs {amount} less if you've sacrificed an artifact this
    // turn" (Suspicious Detonation).
    if let Some(amount) = card.definition.self_cost_reduction_if_sacrificed_artifact
        && state.players[caster].artifacts_sacrificed_this_turn > 0
    {
        reduction = reduction.saturating_add(amount);
    }
    // Card-intrinsic "costs {1} less for each card you've drawn this turn"
    // (Deem Inferior). Generic-only, clamped by the caller.
    if card.definition.self_cost_reduction_per_cards_drawn {
        reduction = reduction.saturating_add(state.players[caster].cards_drawn_this_turn);
    }
    // Card-intrinsic "costs {amount} less if you've cast another spell this
    // turn" (Rally the Monastery). `spells_cast_this_turn` excludes the spell
    // being cast, so `> 0` means a prior spell went off.
    if let Some(amount) = card.definition.self_cost_reduction_if_cast_spell
        && state.players[caster].spells_cast_this_turn > 0
    {
        reduction = reduction.saturating_add(amount);
    }
    // Card-intrinsic predicate-gated discount (the Prophecy Avatars).
    if let Some((cond, amount)) = &card.definition.self_cost_reduction_if {
        let ctx = crate::game::effects::EffectContext::for_spell(caster, None, 0, 0);
        if state.evaluate_predicate(cond, &ctx) {
            reduction = reduction.saturating_add(*amount);
        }
    }
    // Card-intrinsic scaled discount — "{per} less for each [value]" (Domain:
    // Draco, Stratadon). Generic-only, clamped by the caller.
    if let Some((value, per)) = &card.definition.self_cost_reduction_per {
        let ctx = crate::game::effects::EffectContext::for_spell(caster, None, 0, 0);
        let units = state.evaluate_value(value, &ctx).max(0) as u32;
        reduction = reduction.saturating_add(units.saturating_mul(*per));
    }
    // Card-intrinsic "costs {amount} less if evidence was collected" (Bite Down
    // on Crime, CR 701.59). The collect is an optional additional cost announced
    // during casting; the auto-decider collects whenever the graveyard can
    // afford it, so mirror that here to keep cost-calc and payment consistent.
    if let Some(amount) = card.definition.self_cost_reduction_if_collect_evidence
        && card
            .definition
            .additional_cast_cost
            .iter()
            .any(|c| matches!(c, crate::card::AdditionalCastCost::CollectEvidence { amount, .. }
                if state.graveyard_can_collect_evidence(caster, *amount)))
    {
        reduction = reduction.saturating_add(amount);
    }
    // Card-intrinsic "costs {X} less, where X is the greatest power among
    // creatures you control" (The Great Henge) — a `SelfCostReducedByGreatest-
    // Power` static carried by the spell being cast. Generic-only, clamped by
    // the caller via `ManaCost::reduce_generic`.
    if card
        .definition
        .static_abilities
        .iter()
        .any(|sa| matches!(sa.effect, StaticEffect::SelfCostReducedByGreatestPower))
    {
        let greatest = state
            .battlefield
            .iter()
            .filter(|c| c.controller == caster && c.definition.is_creature())
            .map(|c| c.power().max(0) as u32)
            .max()
            .unwrap_or(0);
        reduction = reduction.saturating_add(greatest);
    }
    // Card-intrinsic "costs {X} less, where X is the total power of creatures
    // you control" (Ghalta, Primal Hunger). Generic-only, clamped by the caller.
    if card
        .definition
        .static_abilities
        .iter()
        .any(|sa| matches!(sa.effect, StaticEffect::SelfCostReducedByTotalPower))
    {
        let total: u32 = state
            .battlefield
            .iter()
            .filter(|c| c.controller == caster && c.definition.is_creature())
            .map(|c| c.power().max(0) as u32)
            .sum();
        reduction = reduction.saturating_add(total);
    }
    // Card-intrinsic "costs {1} less for each creature card in your graveyard"
    // (Ghoultree). Generic-only, clamped by the caller.
    if card
        .definition
        .static_abilities
        .iter()
        .any(|sa| matches!(sa.effect, StaticEffect::SelfCostReducedPerCreatureInGraveyard))
    {
        let n = state.players[caster]
            .graveyard
            .iter()
            .filter(|c| c.definition.is_creature())
            .count() as u32;
        reduction = reduction.saturating_add(n);
    }
    // Card-intrinsic "costs {1} less for each card type among cards in your
    // graveyard" (Emrakul, the Promised End). Generic-only, clamped by caller.
    if card
        .definition
        .static_abilities
        .iter()
        .any(|sa| matches!(sa.effect, StaticEffect::SelfCostReducedPerCardTypeInGraveyard))
    {
        let types: std::collections::HashSet<crate::card::CardType> = state.players[caster]
            .graveyard
            .iter()
            .flat_map(|c| c.definition.card_types.iter().cloned())
            .collect();
        reduction = reduction.saturating_add(types.len() as u32);
    }
    // Card-intrinsic "costs {X} less, X = total MV of noncreature artifacts
    // you control" (Metalwork Colossus). Generic-only, clamped by the caller.
    if card
        .definition
        .static_abilities
        .iter()
        .any(|sa| matches!(sa.effect, StaticEffect::SelfCostReducedByNoncreatureArtifactMv))
    {
        let total: u32 = state
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == caster
                    && c.definition.is_artifact()
                    && !c.definition.is_creature()
            })
            .map(|c| c.definition.cost.cmc())
            .sum();
        reduction = reduction.saturating_add(total);
    }
    // Card-intrinsic "costs {per} less for each [filter] card in your
    // graveyard" (Serpent of the Pass). Generic-only, clamped by the caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedPerGraveyardCardMatching { filter, per } = &sa.effect {
            let n = state.players[caster]
                .graveyard
                .iter()
                .filter(|c| state.evaluate_requirement_on_card(filter, c, caster))
                .count() as u32;
            reduction = reduction.saturating_add(n.saturating_mul(*per));
        }
    }
    // Card-intrinsic "costs {per} less for each [filter] permanent you control"
    // — Affinity for [type] (Allies at Last). Generic-only, clamped by caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedPerPermanentMatching { filter, per } = &sa.effect {
            // Evaluate through the battlefield-aware path so board-state filters
            // (IsModified, Tapped, …) resolve — `evaluate_requirement_on_card`
            // treats those as false. Walking Skyscraper counts modified creatures.
            // The implicit "you control" is skipped when the printed filter
            // names a controller itself (Obsidian Charmaw counts *opponents'*
            // colorless lands via `ControlledByOpponent`).
            let filter_names_controller = requirement_mentions_controller(filter);
            let n = state
                .battlefield
                .iter()
                .filter(|c| {
                    (filter_names_controller || c.controller == caster)
                        && state.evaluate_requirement_static(
                            filter,
                            &crate::game::Target::Permanent(c.id),
                            caster,
                            Some(card.id),
                        )
                })
                .count() as u32;
            reduction = reduction.saturating_add(n.saturating_mul(*per));
        }
    }
    // Card-intrinsic "costs {amount} less if a creature died this turn" (Bone
    // Picker). Generic-only, clamped by the caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedIfCreatureDiedThisTurn { amount } = sa.effect
            && state.players.iter().any(|p| p.creatures_died_this_turn > 0)
        {
            reduction = reduction.saturating_add(amount);
        }
    }
    // Card-intrinsic "costs {amount} less if [condition]" (Avatar of Hope).
    // Generic-only, clamped by the caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedIfPredicate { amount, condition } = &sa.effect {
            let mut ctx = crate::game::effects::EffectContext::for_spell(caster, None, 0, 0);
            ctx.source = Some(card.id);
            if state.evaluate_predicate(condition, &ctx) {
                reduction = reduction.saturating_add(*amount);
            }
        }
    }
    // Card-intrinsic "costs {X} less, where X is your Domain" (Leyline Binding)
    // — distinct basic land types among the caster's lands. Generic-only,
    // clamped by the caller via `ManaCost::reduce_generic`.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedByDomain { per } = sa.effect {
            reduction = reduction.saturating_add(per * state.domain_count(caster) as u32);
        }
    }
    // Card-intrinsic "costs {X} less, where X is the number of differently
    // named lands you control" (Fungal Colossus). Generic-only, clamped by the
    // caller via `ManaCost::reduce_generic`.
    if card
        .definition
        .static_abilities
        .iter()
        .any(|sa| matches!(sa.effect, StaticEffect::SelfCostReducedByDistinctLandNames))
    {
        let mut names: Vec<&str> = state
            .battlefield
            .iter()
            .filter(|c| c.controller == caster && c.definition.is_land())
            .map(|c| c.definition.name)
            .collect();
        names.sort_unstable();
        names.dedup();
        reduction = reduction.saturating_add(names.len() as u32);
    }
    // Card-intrinsic "costs {amount} less during your turn" (Mental Modulation).
    // Generic-only, clamped by the caller.
    if state.active_player_idx == caster {
        for sa in &card.definition.static_abilities {
            if let StaticEffect::SelfCostReducedDuringYourTurn { amount } = sa.effect {
                reduction = reduction.saturating_add(amount);
            }
        }
    }
    // Card-intrinsic "costs {X} less, where X is your devotion to [colors]"
    // (Theros — Daybreak Chimera). Generic-only, clamped by the caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedByDevotion { colors } = &sa.effect {
            reduction = reduction.saturating_add(state.devotion_to(caster, colors) as u32);
        }
    }
    // One-shot "the next instant or sorcery you cast this turn costs {N}
    // less" discounts (Thundertrap Trainer). Each was stamped with the
    // caster's instant/sorcery tally at grant time; it applies only while
    // that tally is unchanged — i.e. to the *next* such spell — and then
    // naturally lapses once the tally ticks up on cast (no consume hook
    // needed). The tally increments at spell-commit, after this read.
    if card.definition.is_instant() || card.definition.is_sorcery() {
        let cast_so_far = state.players[caster].instants_or_sorceries_cast_this_turn;
        for &(amount, granted_at) in &state.players[caster].pending_is_discounts {
            if granted_at == cast_so_far {
                reduction = reduction.saturating_add(amount);
            }
        }
    }
    // One-shot "the next spell you cast this turn costs {N} less"
    // discounts (Mutated Cultist) — any spell type; same lapse-by-tally
    // scheme as `pending_is_discounts` above.
    {
        let cast_so_far = state.players[caster].spells_cast_this_turn;
        for &(amount, granted_at) in &state.players[caster].pending_spell_discounts {
            if granted_at == cast_so_far {
                reduction = reduction.saturating_add(amount);
            }
        }
    }
    // Card-intrinsic "costs {N} less if you control a permanent matching each
    // filter" (Of One Mind). Generic-only, clamped by the caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedIfControlEach { filters, amount } = &sa.effect {
            let all_present = filters.iter().all(|f| {
                state
                    .battlefield
                    .iter()
                    .any(|c| c.controller == caster && state.evaluate_requirement_on_card(f, c, caster))
            });
            if all_present {
                reduction = reduction.saturating_add(*amount);
            }
        }
    }
    // Card-intrinsic "costs {N} less if [predicate]" (Gigastorm Titan, Lashwhip
    // Predator). Predicate evaluated with the caster as controller. Generic-only.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedIf { condition, amount } = &sa.effect {
            let ctx = crate::game::effects::EffectContext::for_ability(card.id, caster, None);
            if state.evaluate_predicate(condition, &ctx) {
                reduction = reduction.saturating_add(*amount);
            }
        }
    }
    // Card-intrinsic "costs {N} less per card you've discarded this turn"
    // (Hollow One). Generic-only, clamped by the caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedPerDiscardThisTurn { per } = &sa.effect {
            reduction = reduction
                .saturating_add(per * state.players[caster].cards_discarded_this_turn);
        }
    }
    // CR 702.125 — Undaunted: "costs {N} less to cast for each opponent."
    // Generic-only, clamped by the caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedPerOpponent { per } = &sa.effect {
            let opponents = state.opponents_of(caster).len() as u32;
            reduction = reduction.saturating_add(per * opponents);
        }
    }
    // "Costs {N} less for each other spell cast this turn" (Thrasta) —
    // every player's casts count. Generic-only, clamped by the caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedPerSpellCastThisTurn { per } = &sa.effect {
            let count: u32 = state.players.iter().map(|p| p.spells_cast_this_turn).sum();
            reduction = reduction.saturating_add(per * count);
        }
    }
    // Card-intrinsic "costs {N} less per creature you attacked with this turn"
    // (Search Party Captain). Generic-only, clamped by the caller.
    for sa in &card.definition.static_abilities {
        if let StaticEffect::SelfCostReducedPerCreatureAttackedThisTurn { per, all_players } =
            &sa.effect
        {
            let count: u32 = if *all_players {
                state.players.iter().map(|p| p.creatures_attacked_this_turn).sum()
            } else {
                state.players[caster].creatures_attacked_this_turn
            };
            reduction = reduction.saturating_add(per * count);
        }
    }
    // Turn-scoped "[filter] spells you cast this turn cost {N} less"
    // grants (Urza, Planeswalker's +2). Cleared at cleanup.
    for (filter, amount) in &state.players[caster].turn_spell_discounts {
        if state.evaluate_requirement_on_card(filter, card, caster) {
            reduction = reduction.saturating_add(*amount);
        }
    }
    // Transient "sacrifice any number, {N} less each" additional-cost
    // reduction (Awaken the Blood Avatar). Stamped on the state for the
    // duration of one cast by `cast_spell_sacrifice_reduce`.
    reduction = reduction.saturating_add(state.extra_cast_reduction);
    reduction
}

/// Trinisphere floor: the largest `StaticEffect::SpellCostFloor` amount
/// among untapped battlefield permanents (affects every player's spells).
/// Returns 0 if none in play.
pub(crate) fn spell_cost_floor(state: &crate::game::GameState) -> u32 {
    use crate::effect::StaticEffect;
    state
        .battlefield
        .iter()
        .filter(|c| !c.tapped)
        .flat_map(|c| c.definition.static_abilities.iter())
        .filter_map(|sa| match &sa.effect {
            StaticEffect::SpellCostFloor { amount } => Some(*amount),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

/// Raise `cost` so its mana value is at least the active Trinisphere floor
/// by padding generic mana. No-op when no floor is in play or the cost is
/// already at/above it.
pub(crate) fn apply_spell_cost_floor(
    state: &crate::game::GameState,
    cost: &mut crate::mana::ManaCost,
) {
    let floor = spell_cost_floor(state);
    let mv = cost.cmc();
    if floor > mv {
        cost.symbols
            .push(crate::mana::ManaSymbol::Generic(floor - mv));
    }
}

/// True if `player` controls a permanent granting Omniscience-style free
/// casting of hand spells (`StaticEffect::CastHandSpellsFree`).
impl crate::game::GameState {
    pub(crate) fn player_casts_hand_spells_free(
        &self,
        player: usize,
        card: &crate::card::CardInstance,
    ) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition.static_abilities.iter().any(|sa| match &sa.effect {
                    StaticEffect::CastHandSpellsFree => true,
                    StaticEffect::CastFilteredSpellsFree { filter } => {
                        self.evaluate_requirement_on_card(filter, card, player)
                    }
                    _ => false,
                })
        })
    }

    /// Conspiracy Unraveler — the smallest collect-evidence amount `player`
    /// can substitute for a spell's mana cost, if they control such a static
    /// and their graveyard can actually pay it.
    pub(crate) fn player_casts_spells_for_evidence(&self, player: usize) -> Option<u32> {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == player)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match sa.effect {
                StaticEffect::CastHandSpellsForCollectEvidence { amount } => Some(amount),
                _ => None,
            })
            .filter(|n| self.graveyard_can_collect_evidence(player, *n))
            .min()
    }

    /// Aluren — true if some battlefield permanent grants "any player may
    /// cast creature spells with mana value N or less for free", and `def`
    /// qualifies (a creature within the MV cap). The grant is global, so the
    /// controller of the Aluren is irrelevant.
    pub(crate) fn player_casts_cheap_creature_free(
        &self,
        def: &crate::card::CardDefinition,
    ) -> bool {
        use crate::effect::StaticEffect;
        if !def.card_types.contains(&crate::card::CardType::Creature) {
            return false;
        }
        let mv = def.cost.cmc();
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect,
                    StaticEffect::AnyoneCastsCheapCreaturesFree { max_mv } if mv <= max_mv)
            })
        })
    }
}

/// True if any battlefield permanent has `StaticEffect::LandsTapColorlessOnly`
/// (Damping Sphere). Used by `play_land` to decide whether to downgrade
/// multi-color/multi-mana lands to "{T}: Add {C}".
pub(crate) fn multi_mana_ability_count(def: &crate::card::CardDefinition) -> bool {
    use crate::effect::Effect;
    // The Oracle says "tap to add more than one mana" — for our purposes:
    // any land with two or more separate mana abilities, OR any single
    // ability that produces an `AnyOneColor`/`AnyColors` payload (which
    // could conceptually be more than one), counts. Single-color basics
    // (one ability, `Colors([X])` of length 1) and single-color non-basics
    // (one ability) pass through unchanged.
    let mana_abilities: Vec<_> = def
        .activated_abilities
        .iter()
        .filter(|a| matches!(a.effect, Effect::AddMana { .. }))
        .collect();
    if mana_abilities.len() >= 2 {
        return true;
    }
    if let Some(a) = mana_abilities.first()
        && let Effect::AddMana { pool, .. } = &a.effect
    {
        return payload_yields_multiple(pool);
    }
    false
}

/// True if a single `AddMana` payload could yield more than one mana (or a
/// player-chosen color), used by `multi_mana_ability_count`. A spend
/// restriction is transparent here — it wraps an inner payload whose shape
/// is what matters.
fn payload_yields_multiple(pool: &crate::effect::ManaPayload) -> bool {
    use crate::effect::ManaPayload;
    match pool {
        ManaPayload::AnyOneColor(_)
        | ManaPayload::AnyColors(_)
        | ManaPayload::DevotionOfChosenColor
        | ManaPayload::ImprintedCardColor
        | ManaPayload::AnyColorOpponentCouldProduce
        | ManaPayload::AnyColorYouCouldProduce
        | ManaPayload::AnyTypeTriggerSourceProduces
        | ManaPayload::AnyTypeSacrificedLandProduces
        | ManaPayload::AnyColorAmongLegendaries
        | ManaPayload::AnyColorAmongYourPermanents
        | ManaPayload::DraftNotedColorOfSource => true,
        ManaPayload::Colors(cs) => cs.len() > 1,
        ManaPayload::OfColors(cs, _) => cs.len() > 1,
        ManaPayload::OfColor(_, _)
        | ManaPayload::Colorless(_)
        | ManaPayload::ChosenColorOfSource => false,
        ManaPayload::Restricted(inner, _) | ManaPayload::RestrictedToChosenType(inner)
                    | ManaPayload::RestrictedToChosenTypePlain(inner) => {
            payload_yields_multiple(inner)
        }
    }
}

/// Elesh Norn, Mother of Machines: count how many times an ETB trigger
/// from a permanent owned by `etb_controller` should fire.
///
/// Rules:
/// - "Permanents entering the battlefield don't cause abilities of permanents
///   your opponents control to trigger" → if any opponent of the
///   permanent's controller has an Elesh Norn, the trigger is suppressed
///   (returns 0).
/// - "If a permanent entering the battlefield causes a triggered ability of
///   a permanent you control to trigger, that ability triggers an additional
///   time" → each Elesh Norn on the trigger-source's side adds one extra fire.
///
/// `etb_controller` is the controller of the ability's source — for self-ETB
/// triggers, that's the entering permanent itself.
pub fn etb_trigger_multiplier(
    state: &crate::game::GameState,
    etb_controller: usize,
    entering: Option<CardId>,
) -> usize {
    use crate::effect::StaticEffect;
    // Torpor Orb / Tocatli Honor Guard (CR 614): an entering *creature*
    // causes no triggered abilities to trigger while a suppressor is in play.
    if let Some(id) = entering
        && state
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .is_some_and(|c| c.definition.is_creature())
        && creature_etb_triggers_suppressed(state)
    {
        return 0;
    }
    // Doorkeeper Thrull — an entering *artifact* likewise causes no triggers
    // while an `also_artifacts` suppressor is in play.
    if let Some(id) = entering
        && state
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .is_some_and(|c| c.definition.card_types.contains(&crate::card::CardType::Artifact))
        && state.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(
                    sa.effect,
                    StaticEffect::SuppressCreatureEtbTriggers { also_artifacts: true, .. }
                )
            })
        })
    {
        return 0;
    }
    let mut your_norns = 0usize;
    let mut opp_norns = 0usize;
    // Yarok/Panharmonicon-style doublers add fires for the controller's own
    // ETB triggers without suppressing opponents'.
    let mut your_doublers = 0usize;
    for c in &state.battlefield {
        let count_spotlight = c
            .definition
            .static_abilities
            .iter()
            .filter(|sa| matches!(sa.effect, StaticEffect::EtbTriggerSpotlight))
            .count();
        let count_doubler = c
            .definition
            .static_abilities
            .iter()
            .filter(|sa| matches!(sa.effect, StaticEffect::DoubleControllerEtbTriggers))
            .count();
        if c.controller == etb_controller {
            your_norns += count_spotlight;
            your_doublers += count_doubler;
        } else {
            opp_norns += count_spotlight;
        }
    }
    if opp_norns > 0 {
        0
    } else {
        1 + your_norns + your_doublers
    }
}

/// True when any battlefield permanent carries a
/// `SuppressCreatureEtbTriggers` static (Torpor Orb, Tocatli Honor Guard,
/// Hushbringer). Applies globally — both players' creature ETB triggers.
pub(crate) fn creature_etb_triggers_suppressed(state: &crate::game::GameState) -> bool {
    use crate::effect::StaticEffect;
    state.battlefield.iter().any(|c| {
        c.definition
            .static_abilities
            .iter()
            .any(|sa| matches!(sa.effect, StaticEffect::SuppressCreatureEtbTriggers { .. }))
    })
}

/// CR 603-style "triggers an additional time" — extra fires a triggered
/// ability gets because its `source` is a creature controlled by a player who
/// controls a subtype-keyed trigger doubler: `DoubleControllerAllyTriggers`
/// (Katara, the Fearless — Allies) or the general
/// `DoubleControllerTriggersOfType` (Harmonic Prodigy — Shaman / another
/// Wizard), `DoubleControllerLegendaryCreatureTriggers` (Annie Joins Up) or
/// the unconditional `DoubleControllerPermanentTriggers` (Fractured Realm).
/// 0 unless the source is a matching permanent `controller` controls.
pub(crate) fn ally_trigger_extra_fires(
    state: &crate::game::GameState,
    controller: usize,
    source: crate::card::CardId,
) -> usize {
    use crate::effect::StaticEffect;
    // The triggering source's current creature types (read live off the
    // battlefield; a source no longer in play or not `controller`'s → 0).
    let Some(cp) = state.computed_permanent(source) else { return 0 };
    if state.battlefield_find(source).is_none_or(|c| c.controller != controller) {
        return 0;
    }
    let source_types = cp.subtypes.creature_types.clone();
    let is_ally = source_types.contains(&crate::card::CreatureType::Ally);
    let legendary_creature = cp.card_types.contains(&crate::card::CardType::Creature)
        && cp.supertypes.contains(&crate::card::Supertype::Legendary);
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == controller)
        .map(|c| {
            c.definition
                .static_abilities
                .iter()
                .filter(|sa| match &sa.effect {
                    StaticEffect::DoubleControllerAllyTriggers => is_ally,
                    StaticEffect::DoubleControllerTriggersOfType { types, exclude_source } => {
                        (!*exclude_source || c.id != source)
                            && types.iter().any(|t| source_types.contains(t))
                    }
                    StaticEffect::DoubleControllerTriggersMatching { filter } => state
                        .evaluate_requirement_static(
                            filter,
                            &Target::Permanent(source),
                            controller,
                            Some(c.id),
                        ),
                    StaticEffect::DoubleControllerLegendaryCreatureTriggers => legendary_creature,
                    StaticEffect::DoubleControllerPermanentTriggers => true,
                    _ => false,
                })
                .count()
        })
        .sum()
}

/// True when any battlefield permanent carries a
/// `SuppressCreatureEtbTriggers { also_dies: true }` static (Hushbringer).
/// Suppresses creature-death triggers globally (CR 614).
pub(crate) fn creature_dies_triggers_suppressed(state: &crate::game::GameState) -> bool {
    use crate::effect::StaticEffect;
    state.battlefield.iter().any(|c| {
        c.definition.static_abilities.iter().any(|sa| {
            matches!(sa.effect, StaticEffect::SuppressCreatureEtbTriggers { also_dies: true, .. })
        })
    })
}

/// Strict Proctor ETB-trigger tax — CR 614 replacement effect.
///
/// Strict Proctor — "Whenever a permanent entering causes a triggered
/// ability to trigger, counter that ability unless its controller pays
/// {amount}." Read at ETB-trigger dispatch time for each
/// `StaticEffect::EtbTriggerTax` in play.
///
/// Returns `true` if the trigger should fire (controller paid or no tax in
/// play), `false` if it should be countered (controller declined or
/// couldn't pay). Countering the ability has no other consequence — the
/// trigger simply never fires (CR 701.5a).
///
/// `trigger_source` is the permanent whose ability is triggering (used to
/// aim the pay-or-counter decision at something visible).
pub(crate) fn apply_etb_trigger_tax(
    state: &mut crate::game::GameState,
    trigger_source: crate::card::CardId,
    trigger_controller: usize,
) -> bool {
    use crate::decision::{Decision, DecisionAnswer};
    use crate::effect::StaticEffect;
    use crate::mana::ManaCost;

    // Sum tax amounts from every Strict Proctor on the battlefield.
    // Each Strict Proctor demands its own payment per the printed
    // "counter that ability unless its controller pays {2}" wording —
    // but applied as a single rolled-up amount via additive tax
    // (matching the existing engine's handling of stacking-tax effects).
    let total_tax: u32 = state
        .battlefield
        .iter()
        .flat_map(|c| c.definition.static_abilities.iter())
        .filter_map(|sa| {
            if let StaticEffect::EtbTriggerTax { amount } = &sa.effect {
                Some(*amount)
            } else {
                None
            }
        })
        .sum();
    if total_tax == 0 {
        return true;
    }
    // Build a "Pay {total_tax}" decision aimed at the trigger's controller.
    let answer = state.decider.decide(&Decision::OptionalTrigger {
        source: trigger_source,
        description: format!("Pay {{{}}} to keep this trigger?", total_tax),
    });
    if matches!(answer, DecisionAnswer::Bool(true)) {
        let cost = ManaCost::new(vec![crate::mana::generic(total_tax)]);
        if state.players[trigger_controller].mana_pool.pay(&cost).is_ok() {
            return true;
        }
        // Couldn't actually afford the tax — fall through and counter.
    }
    // Counter the triggered ability (CR 701.5a): it never fires, and
    // nothing else happens — the printed card counters the ability, it
    // does NOT touch the trigger's source permanent.
    false
}

/// Cast-time "can't be countered" checks (printed keyword, turn-scoped
/// grants, Banefire X gates). Cavern of Souls rides mana provenance
/// instead — see `note_cast_payment_riders`.
impl crate::game::GameState {
    /// Legacy entrypoint kept for symmetry; new call sites should use
    /// `caster_grants_uncounterable_with_x` to thread the cast's X
    /// value. Internally delegates with X = 0.
    #[allow(dead_code)]
    pub fn caster_grants_uncounterable(
        &self,
        caster: usize,
        card: &crate::card::CardInstance,
    ) -> bool {
        self.caster_grants_uncounterable_with_x(caster, card, 0)
    }

    /// X-aware variant. Threaded by `finalize_cast` so cards whose
    /// "can't be countered" rider is gated on the X value (Banefire's
    /// "if X is 5 or more, this spell can't be countered") see the
    /// correct flag at cast time.
    pub(crate) fn caster_grants_uncounterable_with_x(
        &self,
        caster: usize,
        card: &crate::card::CardInstance,
        x_value: u32,
    ) -> bool {
        // The card itself is uncounterable (Dovin's Veto, Stubborn Denial,
        // etc. — `Keyword::CantBeCountered`).
        if card.definition.keywords.contains(&Keyword::CantBeCountered) {
            return true;
        }
        // Turn-scoped grant — Veil of Summer's "spells your opponents
        // control can't counter spells you control this turn."
        if self.players[caster].spells_uncounterable_this_turn {
            return true;
        }
        // Creature-spell-only grant (Domri, Anarch of Bolas's +1).
        if self.players[caster].creature_spells_uncounterable_this_turn
            && card.definition.is_creature()
        {
            return true;
        }
        // "The next [filter] spell you cast this turn can't be countered"
        // (Insist, Overmaster). Consumed by `consume_next_spell_uncounterable`
        // once the cast goes through.
        if self.players[caster]
            .next_spell_uncounterable
            .iter()
            .any(|f| self.evaluate_requirement_on_card(f, card, caster))
        {
            return true;
        }
        // Conditional "if X is N or more, this spell can't be countered"
        // rider (Banefire-style). Threshold lives on the card's printed
        // keywords as `CantBeCounteredIfXAtLeast(threshold)`; checked
        // against the actual paid X. Any future card with the same
        // shape plugs in by carrying the keyword — no engine change.
        let xcc_threshold = card.definition.keywords.iter().find_map(|kw| {
            if let Keyword::CantBeCounteredIfXAtLeast(n) = kw {
                Some(*n)
            } else {
                None
            }
        });
        if let Some(n) = xcc_threshold
            && x_value >= n
        {
            return true;
        }
        // Battlefield statics — "creature and enchantment spells you
        // control can't be countered" (Destiny Spinner). The card is mid-cast
        // (in no zone yet), so match the filter on the card itself.
        for src in self.battlefield.iter().filter(|c| c.controller == caster) {
            for sa in &src.definition.static_abilities {
                if let crate::effect::StaticEffect::SpellsUncounterable { filter } = &sa.effect
                    && self.evaluate_requirement_on_card(filter, card, caster)
                {
                    return true;
                }
            }
        }
        // Symmetric "creature spells can't be countered" (Leyline of Lifeforce):
        // any player's copy protects every player's creature spells, so scan the
        // whole battlefield rather than just the caster's permanents.
        if card.definition.is_creature()
            && self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::CreatureSpellsCantBeCountered)
                })
            })
        {
            return true;
        }
        // The filtered sibling, also symmetric (Root Sliver).
        let matching: Vec<crate::card::SelectionRequirement> = self
            .battlefield
            .iter()
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match &sa.effect {
                crate::effect::StaticEffect::SpellsCantBeCounteredMatching { filter } => {
                    Some(filter.clone())
                }
                _ => None,
            })
            .collect();
        // The spell is on the stack, so match against the card itself.
        if matching.iter().any(|f| {
            crate::game::layers::requirement_matches_card(f, card, card.controller)
        }) {
            return true;
        }
        // Cavern of Souls' "can't be countered" rider is provenance-based:
        // it rides the spent mana (`SpendRestriction::
        // CreatureOfTypeUncounterable` → `cast_paid_uncounterable`), not a
        // battlefield scan — see `note_cast_payment_riders`.
        false
    }

    /// Lier, Disciple of the Drowned: if `seat` controls a permanent granting
    /// "each instant and sorcery card in your graveyard has flashback (= its
    /// mana cost)", returns the flashback cost for an I/S `card` that lacks a
    /// printed/granted flashback of its own. Consulted by the flashback-cast
    /// path and the graveyard view.
    pub fn graveyard_flashback_grant(
        &self,
        seat: usize,
        card: &crate::card::CardInstance,
    ) -> Option<crate::mana::ManaCost> {
        if card.effective_flashback().is_some()
            || !(card.definition.is_instant() || card.definition.is_sorcery())
        {
            return None;
        }
        let granted = self.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::GraveyardInstantsSorceriesHaveFlashback
                    )
                })
        });
        // Bösium Strip — "until end of turn, you may cast instant and sorcery
        // spells from the top of your graveyard". The flashback tail already
        // exiles the spell, matching the printed rider.
        let strip = self.players[seat].cast_from_graveyard_top_this_turn
            && self.players[seat].graveyard.last().is_some_and(|c| c.id == card.id);
        (granted || strip).then(|| card.definition.cost.clone())
    }

    /// CR 702.97 — the extra activated abilities a card in `owner`'s graveyard
    /// has by grant (Varolz grants scavenge, cost = mana cost, to that player's
    /// creature cards). Surfaced as virtual `from_graveyard` abilities at
    /// indices ≥ the card's printed activated-ability count. Empty for the
    /// common case.
    pub(crate) fn graveyard_granted_abilities(
        &self,
        owner: usize,
        card: &crate::card::CardInstance,
    ) -> Vec<crate::effect::ActivatedAbility> {
        use crate::effect::StaticEffect;
        let mut out = Vec::new();
        // Instance grants first (Cursecloth Wrappings' until-EOT embalm), so
        // their indices match `granted_abilities_for`'s off-battlefield order.
        out.extend(card.granted_activated_abilities.iter().cloned());
        out.extend(card.granted_activated_eot.iter().cloned());
        // Varolz — scavenge on creature cards, unless the card prints its own.
        if card.definition.is_creature()
            && !card
                .definition
                .activated_abilities
                .iter()
                .any(|a| a.exile_self_cost && a.from_graveyard)
            && self.battlefield.iter().any(|c| {
                c.controller == owner
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::GraveyardCreaturesHaveScavenge))
            })
        {
            out.push(crate::effect::shortcut::scavenge(card.definition.cost.clone()));
        }
        out
    }

    /// The scavenge cost (CR 702.97) a graveyard creature card can be exiled for
    /// — its printed scavenge ability, or a Varolz-style grant — so the client
    /// can offer the activation. `None` for non-scavengers.
    pub fn effective_scavenge_cost(
        &self,
        owner: usize,
        card: &crate::card::CardInstance,
    ) -> Option<crate::mana::ManaCost> {
        let is_scavenge = |ab: &crate::effect::ActivatedAbility| {
            ab.from_graveyard
                && ab.exile_self_cost
                && matches!(
                    &ab.effect,
                    crate::effect::Effect::AddCounter {
                        kind: crate::card::CounterType::PlusOnePlusOne,
                        ..
                    }
                )
        };
        card.definition
            .activated_abilities
            .iter()
            .find(|ab| is_scavenge(ab))
            .map(|ab| ab.mana_cost.clone())
            .or_else(|| {
                self.graveyard_granted_abilities(owner, card)
                    .into_iter()
                    .find(is_scavenge)
                    .map(|ab| ab.mana_cost)
            })
    }

    /// Note restricted-mana riders from a cast's payment: spending
    /// Cavern-of-Souls mana stamps the cast uncounterable (consumed by
    /// `finalize_cast`).
    pub(crate) fn note_cast_payment_riders(
        &mut self,
        receipt: &PaymentReceipt,
        kind: &crate::mana::SpellKind,
    ) {
        use crate::mana::SpendRestriction;
        if receipt.side_effects.spent_restrictions.iter().any(|r| match r {
            SpendRestriction::CreatureOfTypeUncounterable(_) => true,
            // Boseiju — only an instant/sorcery funded this way is stamped.
            SpendRestriction::InstantSorceryUncounterable => kind.instant_or_sorcery,
            _ => false,
        }) {
            self.cast_paid_uncounterable = true;
        }
        // Generator Servant — mana spent on a creature spell grants it haste.
        if kind.creature
            && receipt.side_effects.spent_restrictions.contains(&SpendRestriction::CreatureHaste)
        {
            let p = self.priority.player_with_priority;
            self.players[p].pending_creature_etb_keywords.push(crate::card::Keyword::Haste);
        }
    }

    /// True if any battlefield permanent's static abilities include
    /// `StaticEffect::LandsTapColorlessOnly` (Damping Sphere). Used by
    /// `play_land` to downgrade multi-mana lands to colorless on entry.
    pub(crate) fn lands_tap_colorless_only_active(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::LandsTapColorlessOnly))
        })
    }

    /// CR 305.2 — Number of additional lands `player` may play this turn
    /// over the default "one." Counts every battlefield permanent that
    /// the given player controls whose static abilities include
    /// `StaticEffect::ExtraLandPerTurn` (Exploration, Azusa Lost But
    /// Seeking, Wayward Swordtooth). Each granting permanent adds one,
    /// so two Explorations stack to "three lands per turn."
    pub fn extra_land_plays_per_turn(&self, player: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == player)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::ExtraLandPerTurn))
                    .count() as u32
            })
            .sum()
    }

    /// CR 305.2 — Total lands `player` may play this turn. Defaults to
    /// 1, plus any `ExtraLandPerTurn` static grants, plus the player's
    /// manually-set `extra_land_plays` field (set by resolved effects
    /// like Explore).
    pub fn max_lands_per_turn(&self, player: usize) -> u32 {
        1 + self.extra_land_plays_per_turn(player) + self.players[player].extra_land_plays
    }

    /// CR 305.2a — Whether `player` may legally play another land this
    /// turn. Compares lands already played to the active per-turn cap
    /// (which honors `ExtraLandPerTurn` static effects).
    pub fn can_player_play_land(&self, player: usize) -> bool {
        // CR 305.1 — "You can't play lands" (Aggressive Mining) is absolute.
        if self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect, crate::effect::StaticEffect::NoPlayerCanPlayLands)
                    || (c.controller == player
                        && matches!(
                            sa.effect,
                            crate::effect::StaticEffect::ControllerCantPlayLands
                        ))
            })
        }) {
            return false;
        }
        // Turf Wound — a turn-scoped, player-scoped lock.
        if self.players[player].cant_play_lands_this_turn {
            return false;
        }
        // Damping Engine — the player ahead on permanents can't play lands.
        if self.damping_engine_locks(player) {
            return false;
        }
        self.players[player].lands_played_this_turn < self.max_lands_per_turn(player)
    }

    /// Damping Engine (CR 611) — true when `player` controls strictly more
    /// permanents than each other player and a Damping Engine they haven't
    /// bought out of this turn is on the battlefield.
    pub(crate) fn damping_engine_locks(&self, player: usize) -> bool {
        let count = |seat: usize| {
            self.battlefield.iter().filter(|c| c.controller == seat).count()
        };
        let mine = count(player);
        if (0..self.players.len()).any(|s| s != player && count(s) >= mine) {
            return false;
        }
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::MostPermanentsCantPlay))
                && !self.players[player].statics_ignored_this_turn.contains(&c.id)
        })
    }

    /// CR 402.2 — `player`'s effective maximum hand size, honoring any
    /// `StaticEffect::NoMaximumHandSize` permanent they control (Reliquary
    /// Tower, Thought Vessel). `None` means "no maximum" (skip cleanup
    /// discard entirely).
    pub fn effective_max_hand_size(&self, player: usize) -> Option<usize> {
        use crate::effect::StaticEffect;
        let no_max = self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::NoMaximumHandSize))
        });
        if no_max {
            return None;
        }
        // Jin-Gitaxias — "each opponent's maximum hand size is reduced by N."
        let reduction: usize = self
            .battlefield
            .iter()
            .filter(|c| !self.same_team(c.controller, player))
            .flat_map(|c| c.definition.static_abilities.iter())
            .map(|sa| match sa.effect {
                StaticEffect::OpponentsMaxHandSizeReduced(n) => n as usize,
                _ => 0,
            })
            .sum::<usize>()
            // Thought Nibbler — the controller-scoped reduction.
            + self
                .battlefield
                .iter()
                .filter(|c| c.controller == player)
                .flat_map(|c| c.definition.static_abilities.iter())
                .map(|sa| match sa.effect {
                    StaticEffect::ControllerMaxHandSizeReduced(n) => n as usize,
                    _ => 0,
                })
                .sum::<usize>();
        // Set-to-N overrides (Necrodominance) plus Cursed Rack's chosen-player
        // cap. CR 613.11 — game-rule-modifying effects apply in timestamp
        // order, so the most recently established cap wins, not the smallest.
        let set_to: Option<usize> = self
            .battlefield
            .iter()
            .flat_map(|c| c.definition.static_abilities.iter().map(move |sa| (c, sa)))
            .filter_map(|(c, sa)| match sa.effect {
                StaticEffect::ControllerMaxHandSize(n) if c.controller == player => {
                    Some((c.battlefield_timestamp, n as usize))
                }
                StaticEffect::ChosenPlayerMaxHandSize(n) if c.chosen_player == Some(player) => {
                    Some((c.battlefield_timestamp, n as usize))
                }
                _ => None,
            })
            .max_by_key(|(ts, _)| *ts)
            .map(|(_, n)| n);
        // Minamo Scrollkeeper / Trusted Advisor — "your maximum hand size is
        // increased by N"; copies stack, applied after any set-to override.
        let increase: usize = self
            .battlefield
            .iter()
            .filter(|c| c.controller == player)
            .flat_map(|c| c.definition.static_abilities.iter())
            .map(|sa| match sa.effect {
                StaticEffect::ControllerMaxHandSizeIncreased(n) => n as usize,
                _ => 0,
            })
            .sum();
        let base = set_to.or(self.players[player].max_hand_size);
        base.map(|m| (m + increase).saturating_sub(reduction))
    }

    /// CR 305 — Whether `player` may play lands from their graveyard
    /// (Crucible of Worlds, Ramunap Excavator) via a
    /// `StaticEffect::MayPlayLandsFromGraveyard` permanent.
    pub fn player_may_play_lands_from_graveyard(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        if self.players[player].play_from_graveyard_this_turn {
            return true;
        }
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| {
                        matches!(sa.effect, StaticEffect::MayPlayLandsFromGraveyard)
                            || (self.active_player_idx == player
                                && matches!(
                                    sa.effect,
                                    StaticEffect::PlayCardsFromGraveyardDuringYourTurn
                                ))
                    })
        })
    }

    /// CR 701.10f / 614.5 — combined mana-production multiplier from the
    /// `ManaProductionDoubled` (Mana Reflection) and `ManaProductionTripled`
    /// (Nyxbloom Ancient) permanents `player` controls. Replacements compose
    /// multiplicatively: 2^doublers × 3^triplers, clamped to keep pip math
    /// sane.
    pub fn mana_production_multiplier_for(&self, player: usize) -> u32 {
        use crate::effect::StaticEffect;
        let mut mult: u32 = 1;
        for c in self.battlefield.iter().filter(|c| c.controller == player) {
            for sa in &c.definition.static_abilities {
                match sa.effect {
                    StaticEffect::ManaProductionDoubled => mult = mult.saturating_mul(2),
                    StaticEffect::ManaProductionTripled => mult = mult.saturating_mul(3),
                    _ => {}
                }
            }
        }
        mult.min(1 << 16)
    }

    /// Whether `cost` still has an unpaid `color` pip after `p`'s current
    /// floating pool is applied. Drives the CR 702.51 convoke choice: a
    /// tapped creature contributes a colored pip only where the cost wants
    /// one, otherwise {1}.
    fn cost_still_needs_color(
        &self,
        p: usize,
        cost: &crate::mana::ManaCost,
        color: ManaColor,
    ) -> bool {
        let want = cost
            .symbols
            .iter()
            .filter(|s| {
                matches!(s, crate::mana::ManaSymbol::Colored(c) if *c == color)
                    || matches!(s, crate::mana::ManaSymbol::Hybrid(a, b) if *a == color || *b == color)
            })
            .count() as u32;
        want > self.players[p].mana_pool.amount(color)
    }

    /// Needs-aware "any color" pick for a mana ability that can't suspend for
    /// a real choice: the heaviest colored pip across `p`'s hand — this mana
    /// exists to cast things. (A bare `ChooseColor` ask hits AutoDecider and
    /// always yields White, wasting the pip for any non-white deck;
    /// interactive choice is a TODO.md item.)
    pub(crate) fn best_color_for_hand(&self, p: usize) -> ManaColor {
        let mut best = (0u32, ManaColor::White);
        for c in [
            ManaColor::White,
            ManaColor::Blue,
            ManaColor::Black,
            ManaColor::Red,
            ManaColor::Green,
        ] {
            let pips: u32 = self.players[p]
                .hand
                .iter()
                .map(|h| crate::draft::colored_pip_count(&h.definition.cost, c))
                .sum();
            if pips > best.0 {
                best = (pips, c);
            }
        }
        best.1
    }

    /// Colored-pip weight of everything `p` holds and controls, per color.
    /// Feeds the needs-aware CR 612 text-change picks (and any other auto
    /// "name a color" that should key on what a seat is actually playing).
    fn color_weights(&self, p: usize) -> [(ManaColor, u32); 5] {
        let mut out = [
            (ManaColor::White, 0),
            (ManaColor::Blue, 0),
            (ManaColor::Black, 0),
            (ManaColor::Red, 0),
            (ManaColor::Green, 0),
        ];
        let costs = self.players[p]
            .hand
            .iter()
            .chain(self.battlefield.iter().filter(|c| c.controller == p))
            .map(|c| &c.definition.cost);
        for cost in costs {
            for (color, weight) in out.iter_mut() {
                *weight += crate::draft::colored_pip_count(cost, *color);
            }
        }
        out
    }

    /// The color `p` is most invested in (ties break in WUBRG order).
    pub(crate) fn densest_color_of(&self, p: usize) -> ManaColor {
        let w = self.color_weights(p);
        w.iter().max_by_key(|(_, n)| *n).map(|(c, _)| *c).unwrap_or(ManaColor::White)
    }

    /// The color `p` is least invested in — what you want an opponent's
    /// protection or land type rewritten *to*.
    pub(crate) fn sparsest_color_of(&self, p: usize) -> ManaColor {
        let w = self.color_weights(p);
        w.iter().min_by_key(|(_, n)| *n).map(|(c, _)| *c).unwrap_or(ManaColor::White)
    }

    /// The color `p`'s opponents are most invested in — what you want your own
    /// creature's protection rewritten *to*.
    pub(crate) fn densest_color_among_opponents(&self, p: usize) -> ManaColor {
        let mut totals = [
            (ManaColor::White, 0u32),
            (ManaColor::Blue, 0),
            (ManaColor::Black, 0),
            (ManaColor::Red, 0),
            (ManaColor::Green, 0),
        ];
        for seat in 0..self.players.len() {
            if self.same_team(seat, p) {
                continue;
            }
            for (i, (_, n)) in self.color_weights(seat).iter().enumerate() {
                totals[i].1 += n;
            }
        }
        totals.iter().max_by_key(|(_, n)| *n).map(|(c, _)| *c).unwrap_or(ManaColor::White)
    }

    /// The color words a permanent actually prints (CR 612.2) — today the
    /// `Protection(color)` keywords, the only place a color word appears in a
    /// modeled rules text.
    pub(crate) fn printed_color_words(&self, cid: CardId) -> Vec<ManaColor> {
        self.battlefield
            .iter()
            .find(|c| c.id == cid)
            .map(|c| {
                c.definition
                    .keywords
                    .iter()
                    .filter_map(|kw| match kw {
                        crate::card::Keyword::Protection(col) => Some(*col),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// CR 605.1b — resolve `ExtraManaOnLandTap` triggered mana abilities
    /// after `land_id` (controlled by `p`) was tapped for mana. `resolved`
    /// is the tapping ability's event batch — `Mirror` reads the produced
    /// color off its `ManaAdded` events.
    fn resolve_extra_mana_on_land_tap(
        &mut self,
        land_id: crate::card::CardId,
        p: usize,
        resolved: &[GameEvent],
        events: &mut Vec<GameEvent>,
    ) {
        use crate::effect::{ExtraManaKind, StaticEffect};
        let Some(land) = self.battlefield.iter().find(|c| c.id == land_id) else { return };
        if !land.definition.is_land() {
            return;
        }
        let land = land.clone();
        let grants: Vec<(crate::card::CardId, ExtraManaKind)> = self
            .battlefield
            .iter()
            .flat_map(|src| src.definition.static_abilities.iter().map(move |sa| (src, sa)))
            .filter_map(|(src, sa)| {
                let StaticEffect::ExtraManaOnLandTap { enchanted_only, filter, extra, while_monarch } =
                    &sa.effect
                else {
                    return None;
                };
                if *enchanted_only && src.attached_to != Some(land_id) {
                    return None;
                }
                if *while_monarch && self.monarch != Some(src.controller) {
                    return None;
                }
                (crate::game::layers::requirement_matches_card(filter, &land, src.controller))
                    .then_some((src.id, *extra))
            })
            .collect();
        // Bubbling Muck — the turn-scoped floating version of the same grant.
        let mut grants = grants;
        for (land_type, color) in self.extra_mana_on_land_tap_this_turn.clone() {
            if land.definition.subtypes.land_types.contains(&land_type) {
                grants.push((land_id, ExtraManaKind::Fixed(color)));
            }
        }
        for (src_id, extra) in grants {
            // Colorless-only mirror: fires only when the tap produced {C}.
            if matches!(extra, ExtraManaKind::MirrorColorless) {
                if resolved.iter().any(|e| matches!(e,
                    GameEvent::ColorlessManaAdded { player, .. } if *player == p))
                {
                    self.players[p].mana_pool.add_colorless(1);
                    events.push(GameEvent::ColorlessManaAdded { player: p, source: Some(src_id) });
                }
                continue;
            }
            let color = match extra {
                ExtraManaKind::Fixed(c) => Some(c),
                ExtraManaKind::ChosenColor => self
                    .battlefield_find(src_id)
                    .and_then(|c| c.chosen_color),
                ExtraManaKind::Mirror => resolved.iter().find_map(|e| match e {
                    GameEvent::ManaAdded { player, color, .. } if *player == p => Some(*color),
                    _ => None,
                }),
                ExtraManaKind::FixedPerCreatureType(c, ct) => {
                    let n = self
                        .battlefield
                        .iter()
                        .filter(|x| {
                            x.definition.is_creature()
                                && x.definition.subtypes.creature_types.contains(&ct)
                        })
                        .count();
                    for _ in 0..n {
                        self.players[p].mana_pool.add(c, 1);
                        events.push(GameEvent::ManaAdded {
                            player: p,
                            color: c,
                            source: Some(src_id),
                        });
                    }
                    continue;
                }
                ExtraManaKind::AnyColors(n) => {
                    for _ in 0..n {
                        let c = self.best_color_for_hand(p);
                        self.players[p].mana_pool.add(c, 1);
                        events.push(GameEvent::ManaAdded {
                            player: p,
                            color: c,
                            source: Some(src_id),
                        });
                    }
                    continue;
                }
                ExtraManaKind::AnyColor => Some(self.best_color_for_hand(p)),
                // Handled above (colorless-only fast path).
                ExtraManaKind::MirrorColorless => continue,
            };
            match color {
                Some(c) => {
                    self.players[p].mana_pool.add(c, 1);
                    events.push(GameEvent::ManaAdded { player: p, color: c, source: Some(src_id) });
                }
                // Mirror of a colorless-only production (or no pip found).
                None => {
                    if resolved.iter().any(|e| matches!(e,
                        GameEvent::ColorlessManaAdded { player, .. } if *player == p))
                    {
                        self.players[p].mana_pool.add_colorless(1);
                        events.push(GameEvent::ColorlessManaAdded { player: p, source: Some(src_id) });
                    }
                }
            }
        }
    }
}

impl GameState {
    /// The colours a permanent's mana abilities can produce (Mana Web's
    /// "could produce any type of mana that land could produce").
    pub(crate) fn colors_produced_by(&self, id: CardId) -> Vec<ManaColor> {
        let Some(c) = self.battlefield_find(id) else { return Vec::new() };
        ManaColor::ALL
            .iter()
            .copied()
            .filter(|col| {
                c.definition
                    .activated_abilities
                    .iter()
                    .any(|a| is_mana_ability(&a.effect) && effect_produces_color(&a.effect, *col))
            })
            .collect()
    }
}

fn effect_produces_color(effect: &Effect, color: ManaColor) -> bool {
    match effect {
        Effect::AddMana { pool, .. } => match pool {
            ManaPayload::Colors(cs) => cs.contains(&color),
            ManaPayload::AnyOneColor(_)
            | ManaPayload::AnyColors(_)
            | ManaPayload::AnyColorOpponentCouldProduce
            | ManaPayload::AnyColorYouCouldProduce => true,
            // Color set depends on live board state — not auto-tapped.
            ManaPayload::AnyColorAmongLegendaries
            | ManaPayload::AnyColorAmongYourPermanents
            | ManaPayload::AnyTypeTriggerSourceProduces
            | ManaPayload::AnyTypeSacrificedLandProduces => false,
            // Devotion-scaled: it can make `color`, but only the controller
            // should choose to tap it (devotion may be 0). Not auto-tapped.
            ManaPayload::DevotionOfChosenColor => false,
            ManaPayload::OfColor(c, _) => *c == color,
            ManaPayload::OfColors(cs, _) => cs.contains(&color),
            ManaPayload::Colorless(_) => false,
            // Spend-restricted sources are not auto-tapped: their mana can
            // only fund some spells, so tapping one to "cover" a colored
            // pip could strand an otherwise-payable cast. The controller
            // activates them deliberately (or they float via a trigger),
            // and `pay_for_spell` consumes the floated mana.
            ManaPayload::Restricted(_, _)
            | ManaPayload::RestrictedToChosenType(_)
            | ManaPayload::RestrictedToChosenTypePlain(_) => false,
            // Instance-dependent (the chosen color isn't known at the
            // definition level), so it's not part of the static auto-tap
            // signature; the controller taps it deliberately.
            ManaPayload::ChosenColorOfSource => false,
            // Same for a draft-noted palette (Paliano) — it depends on the
            // seat's note table, not the printed card.
            ManaPayload::DraftNotedColorOfSource => false,
            // Imprinted-card colors aren't known statically (depend on the
            // exiled card), so not auto-tapped; tapped deliberately.
            ManaPayload::ImprintedCardColor => false,
        },
        Effect::Seq(steps) => steps.iter().any(|s| effect_produces_color(s, color)),
        Effect::If { then, else_, .. } => {
            effect_produces_color(then, color) || effect_produces_color(else_, color)
        }
        _ => false,
    }
}

impl GameState {
    // ── Play land ─────────────────────────────────────────────────────────────

    /// Whether `p` holds an outstanding `may_play_until` permission on an
    /// exiled `card_id`. The cast path reads the permission off the card
    /// directly; land plays go through here so "you may play that card"
    /// grants (impulse exile) cover lands as well as spells.
    pub(crate) fn may_play_grant_for(&self, p: usize, card_id: CardId) -> bool {
        self.exile
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| c.may_play_until)
            .is_some_and(|perm| perm.player == p)
    }

    pub(crate) fn play_land(&mut self, card_id: CardId) -> Result<Vec<GameEvent>, GameError> {
        self.play_land_with_face(card_id, /* back_face */ false)
    }

    /// Shared implementation for `PlayLand` and `PlayLandBack`. When
    /// `back_face` is true and the card has a `back_face`, the card's
    /// definition is swapped to the back face's definition before placing on
    /// the battlefield — so the resulting permanent has the back face's
    /// types, mana abilities, and ETB triggers.
    pub(crate) fn play_land_with_face(
        &mut self,
        card_id: CardId,
        back_face: bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // CR 305.2 — honor ExtraLandPerTurn static grants (Exploration,
        // Azusa, Wayward Swordtooth) when checking the per-turn cap.
        if !self.can_player_play_land(p) {
            return Err(GameError::AlreadyPlayedLand);
        }
        // City in a Bottle — a symmetric play-lock binds land plays too.
        if let Some(c) = self.find_card_anywhere(card_id)
            && self.play_locked_for_all(p, c)
        {
            return Err(GameError::SpellNameLocked);
        }
        // Cornered Market — a nonbasic land sharing a nontoken permanent's
        // name can't be played.
        if !self.find_card_anywhere(card_id).is_some_and(|c| c.definition.is_basic())
            && self.name_locked_by_a_permanent(Some(card_id))
        {
            return Err(GameError::AlreadyPlayedLand);
        }
        // CR 401.6 — a PlayFromLibraryTop static (Courser of Kruphix,
        // Oracle of Mul Daya) lets the land be played off the library top.
        let from_top = !self.players[p].has_in_hand(card_id)
            && self.library_top_playable(p, card_id);
        let from_top_capped = from_top && self.library_top_cast_is_capped(p, card_id);
        // CR 118.x — "you may play that card" grants cover lands too. An
        // impulse-exiled land (Light Up the Stage, Gonti Night Minister,
        // Chandra Torch of Defiance) is played from exile, not cast.
        // CR 715.3d — a card whose permanent half is a land is *played* out of
        // adventure exile, not cast (the FIN Town // Adventure cycle).
        let from_adventure = !from_top
            && !self.players[p].has_in_hand(card_id)
            && self.exile.iter().any(|c| {
                c.id == card_id && c.on_adventure && c.owner == p && c.definition.is_land()
            });
        let from_exile = !from_top
            && !self.players[p].has_in_hand(card_id)
            && (from_adventure || self.may_play_grant_for(p, card_id));
        let mut card = if from_top {
            self.players[p].library.remove(0)
        } else if self.players[p].has_in_hand(card_id) {
            self.players[p].remove_from_hand(card_id).unwrap()
        } else if from_exile {
            Self::take_card(&mut self.exile, card_id)
                .ok_or(GameError::CardNotInHand(card_id))?
        } else {
            return Err(GameError::CardNotInHand(card_id));
        };
        let restore = |state: &mut Self, card: crate::card::CardInstance| {
            if from_top {
                state.players[p].library.insert(0, card);
            } else if from_exile {
                state.exile.push(card);
            } else {
                state.players[p].hand.push(card);
            }
        };
        if back_face {
            // Swap to the back face's definition. Reject if there isn't one.
            let Some(back) = card.definition.back_face.clone() else {
                restore(self, card);
                return Err(GameError::NotALand(card_id));
            };
            // Keep the front installed until the play is accepted — a
            // rejected play must restore the card unmodified.
            let front = card.definition.clone();
            card.definition = std::sync::Arc::new(*back);
            if !card.definition.is_land() {
                card.definition = front;
                restore(self, card);
                return Err(GameError::NotALand(card_id));
            }
        } else if !card.definition.is_land() {
            restore(self, card);
            return Err(GameError::NotALand(card_id));
        }
        if from_exile {
            // The permission is consumed by the play (CR 608.2 — a one-shot
            // permission doesn't survive the card changing zones anyway).
            // Applied only once the play is accepted, so a rejected play
            // restores the card unmodified.
            card.may_play_until = None;
            card.granted_alt_cast_cost_eot = None;
            card.face_down = false;
            card.on_adventure = false;
            card.adventuring = false;
        }
        if from_top_capped {
            self.players[p].cast_from_library_top_this_turn = true;
        }
        self.place_land_card(p, card)
    }

    /// CR 305 — Play a land from the controller's graveyard, legal only while
    /// a `StaticEffect::MayPlayLandsFromGraveyard` permanent (Crucible of
    /// Worlds, Ramunap Excavator) is in play. Honors the same sorcery-speed
    /// and one-land-per-turn restrictions as a hand land play.
    /// CR 702.139 — pay {3} at sorcery speed to move a companion from the
    /// sideboard to its owner's hand (once per game; it leaves the
    /// sideboard for good).
    pub(crate) fn companion_to_hand(
        &mut self,
        card_id: CardId,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if !self.players[p]
            .sideboard
            .iter()
            .any(|c| c.id == card_id && c.definition.keywords.contains(&Keyword::Companion))
        {
            return Err(GameError::CardNotInHand(card_id));
        }
        let cost = crate::mana::cost(&[crate::mana::generic(3)]);
        let snapshot = self.snapshot_payment_state(p);
        let forced_only = self.players[p].manual_mana;
        let receipt =
            self.try_pay_after_snapshot_mode(p, &cost, snapshot, forced_only, &crate::mana::SpellKind::default(), None)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        // Re-locate by id at removal time: payment ran between the scan
        // above and this remove and could touch the sideboard.
        let card = Self::take_card(&mut self.players[p].sideboard, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let cid = card.id;
        self.players[p].hand.push(card);
        self.players[p].last_drawn_card = Some(cid);
        Ok(vec![GameEvent::CardDrawn { player: p, card_id: cid }])
    }

    pub(crate) fn play_land_from_graveyard(
        &mut self,
        card_id: CardId,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if !self.can_player_play_land(p) {
            return Err(GameError::AlreadyPlayedLand);
        }
        if !self.player_may_play_lands_from_graveyard(p) {
            return Err(GameError::CardNotInHand(card_id));
        }
        if !self.players[p]
            .graveyard
            .iter()
            .any(|c| c.id == card_id && c.definition.is_land())
        {
            return Err(GameError::NotALand(card_id));
        }
        let card = Self::take_card(&mut self.players[p].graveyard, card_id)
            .ok_or(GameError::NotALand(card_id))?;
        self.entered_from_graveyard_this_turn.insert(card_id);
        self.place_land_card(p, card)
    }

    /// Shared land-placement tail for `play_land_with_face` and
    /// `play_land_from_graveyard`: applies Damping Sphere mana downgrades,
    /// pushes the card to the battlefield, fires ETB triggers, and returns
    /// the land-played events.
    fn place_land_card(
        &mut self,
        p: usize,
        mut card: crate::card::CardInstance,
    ) -> Result<Vec<GameEvent>, GameError> {
        let card_id = card.id;
        // Damping Sphere: if any battlefield permanent grants
        // `LandsTapColorlessOnly`, downgrade this land's mana abilities
        // to `{T}: Add {C}` if the original would have produced more than
        // one mana per tap. Applied in-place on the new instance's
        // definition before the card lands on the battlefield, so all
        // downstream activations see the replaced ability set.
        if self.lands_tap_colorless_only_active()
            && multi_mana_ability_count(&card.definition)
        {
            // Drop the printed mana abilities and replace them with a
            // single `{T}: Add {C}`. Non-mana activated abilities
            // (fetchland sac, manland animate, channel, cycling) survive
            // — Damping Sphere's Oracle only affects mana production.
            let mut kept: Vec<crate::card::ActivatedAbility> = card
                .definition
                .activated_abilities
                .iter()
                .filter(|a| !is_mana_ability(&a.effect))
                .cloned()
                .collect();
            kept.push(crate::card::ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: crate::mana::ManaCost::default(),
                effect: crate::effect::Effect::AddMana {
                    who: crate::effect::PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(
                        crate::effect::Value::Const(1),
                    ),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false,
                exile_other_filter: None,
                self_counter_cost_reduction: None, sac_other_filter: None,
                tap_other_filter: None, from_hand: false,
                ..Default::default()
            });
            std::sync::Arc::make_mut(&mut card.definition).activated_abilities = kept;
        }
        self.players[p].lands_played_this_turn += 1;
        self.battlefield.push(card);
        // Fire self-source ETB triggers for the land (shockland pay-or-tap,
        // surveil-land tap-and-surveil, etc.). The cast path inlines the same
        // logic in `resolve_top_of_stack`; play_land needs an analogous push
        // so triggered abilities on lands actually fire.
        self.fire_self_etb_triggers(card_id, p);
        // CR 614.1c — a printed "enters with N counters" land (the MMQ
        // depletion cycle) gets them off the land drop too.
        let mut counter_events = Vec::new();
        self.apply_printed_etb_counters(card_id, &mut counter_events);
        // CR 714.2b — a Saga land (Urza's Saga) enters with its first lore
        // counter; chapter I fires off the land drop too.
        if self
            .battlefield
            .iter()
            .any(|c| c.id == card_id && !c.definition.saga_chapters.is_empty())
        {
            self.saga_enter_advance(card_id);
        }
        let mut out =
            vec![GameEvent::LandPlayed { player: p, card_id, played: true }];
        out.append(&mut counter_events);
        out.push(GameEvent::PermanentEntered { card_id });
        Ok(out)
    }

    /// Push the source-itself ETB triggered abilities for a permanent that
    /// has just entered the battlefield. Used by `play_land` and by Move →
    /// Battlefield zone changes so triggered abilities fire consistently
    /// regardless of how the permanent arrived.
    /// CR 614.13 — apply "[permanents] enter the battlefield tapped"
    /// replacement effects from *other* permanents to a permanent that has
    /// just entered. Powers Authority of the Consuls / Imposing Sovereign /
    /// Thalia, Heretic Cathar (opponents' creatures) and the symmetric
    /// land-tappers (Root Maze, Kismet). Taps the entrant if any active
    /// `StaticEffect::EntersTapped` whose `applies_to` selector matches it is
    /// in play. Called from [`fire_self_etb_triggers`], the universal "a
    /// permanent entered" hook, so every enter path (cast, token, reanimate,
    /// land drop) is covered.
    /// CR 614 — a permanent printed `enters_under_opponent_control` enters
    /// under an opponent of its controller's choice. Applied at the entry hook,
    /// before any ETB trigger reads a controller. With one opponent alive the
    /// choice is forced; with several the controller picks the first alive
    /// opponent in seat order (a bot-policy default, like the other
    /// auto-resolved entry choices).
    pub(crate) fn apply_enters_under_opponent_control(&mut self, card_id: CardId) {
        let Some(c) = self.battlefield_find(card_id) else { return };
        // The guard keeps the replacement one-shot: every entry path funnels
        // through here, and once applied the controller is no longer the owner.
        if !c.definition.enters_under_opponent_control || c.controller != c.owner {
            return;
        }
        let owner = c.controller;
        let Some(victim) = (0..self.players.len())
            .find(|&p| p != owner && self.players[p].is_alive() && !self.same_team(p, owner))
        else {
            return;
        };
        if let Some(c) = self.battlefield_find_mut(card_id) {
            c.controller = victim;
        }
    }

    pub(crate) fn apply_enters_tapped_replacement(&mut self, card_id: CardId) {
        use crate::effect::StaticEffect;
        use crate::game::layers::{AffectedPermanents, affected_includes};
        let Some(idx) = self.battlefield.iter().position(|c| c.id == card_id) else {
            return;
        };
        // CR 614 — "permanents enter tapped this turn" (Due Respect).
        let mut should_tap = self.permanents_enter_tapped_this_turn;
        // A permanent's own `EntersTapped { applies_to: This/Source }` (e.g.
        // Overlord of the Hauntwoods' "Everywhere" land token) taps itself —
        // the cross-permanent loop below skips the entering card, so handle the
        // self case up front. `EntersTappedUnless` (Horned Loch-Whale) taps
        // itself unless its predicate holds, evaluated with the entrant as
        // source/controller.
        let self_seat = self.battlefield[idx].controller;
        let self_id = self.battlefield[idx].id;
        for sa in &self.battlefield[idx].definition.static_abilities {
            match &sa.effect {
                StaticEffect::EntersTapped { applies_to: crate::effect::Selector::This } => {
                    should_tap = true;
                    break;
                }
                StaticEffect::EntersTappedUnless {
                    applies_to: crate::effect::Selector::This,
                    condition,
                } => {
                    let mut ctx = crate::game::effects::EffectContext::for_ability(
                        self_id, self_seat, None,
                    );
                    // X-conditional enters-tapped ("enters tapped if X is 2
                    // or less" — Slumbering Trudge): the cast's X was
                    // stamped on the instance at resolution, thread it so
                    // `Value::XFromCost` predicates read the real X.
                    ctx.x_value = self.battlefield[idx].cast_x_value;
                    if !self.evaluate_predicate(condition, &ctx) {
                        should_tap = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        for src in &self.battlefield {
            if src.id == card_id {
                continue;
            }
            if should_tap {
                break;
            }
            for sa in &src.definition.static_abilities {
                let StaticEffect::EntersTapped { applies_to } = &sa.effect else {
                    continue;
                };
                let Some(mut affected) = super::selector_to_affected(applies_to, src) else {
                    continue;
                };
                // Team-aware: fill `friendly_seats` like gather_continuous_effects.
                if let AffectedPermanents::AllOpponents { source_controller, friendly_seats, .. } =
                    &mut affected
                    && friendly_seats.is_empty()
                {
                    let mut seats = self.teammates(*source_controller);
                    seats.push(*source_controller);
                    *friendly_seats = seats;
                }
                if affected_includes(&affected, src.id, &self.battlefield[idx]) {
                    should_tap = true;
                    break;
                }
            }
            if should_tap {
                break;
            }
        }
        // CR 614 — an "enters untapped" replacement (Spelunking) overrides the
        // enters-tapped effects for lands the static-source's controller owns.
        if should_tap && self.battlefield[idx].definition.is_land() {
            let entrant_controller = self.battlefield[idx].controller;
            let overridden = self.battlefield.iter().any(|src| {
                src.controller == entrant_controller
                    && src
                        .definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::LandsEnterUntapped))
            });
            if overridden {
                should_tap = false;
            }
        }
        if should_tap {
            self.battlefield[idx].tapped = true;
        }
    }

    /// CR 616.1c / 616.1g — the "enters as a copy" replacement is applied
    /// *before* the enters-tapped one, so the entrant enters tapped only if its
    /// **copied** characteristics say so (Essence of the Wild's copy of Rusted
    /// Sentinel loses the printed enters-tapped ability and enters untapped).
    /// The engine applies enters-tapped first, so re-decide once a copy lands.
    pub(crate) fn reapply_enters_tapped_after_copy(&mut self, card_id: CardId) {
        if let Some(c) = self.battlefield_find_mut(card_id) {
            c.tapped = false;
        }
        self.apply_enters_tapped_replacement(card_id);
    }

    /// CR 704.5g (Zilortha) — true iff some active `LethalDamageByPower` static
    /// matches the creature `card_id`, so its lethal-damage threshold is power
    /// rather than toughness.
    pub(crate) fn lethal_damage_by_power(&self, card_id: CardId) -> bool {
        use crate::effect::StaticEffect;
        use crate::game::layers::{AffectedPermanents, affected_includes};
        let Some(target) = self.battlefield_find(card_id) else { return false };
        for src in &self.battlefield {
            for sa in &src.definition.static_abilities {
                let StaticEffect::LethalDamageByPower { applies_to } = &sa.effect else {
                    continue;
                };
                let Some(mut affected) = super::selector_to_affected(applies_to, src) else {
                    continue;
                };
                if let AffectedPermanents::AllOpponents { source_controller, friendly_seats, .. } =
                    &mut affected
                    && friendly_seats.is_empty()
                {
                    let mut seats = self.teammates(*source_controller);
                    seats.push(*source_controller);
                    *friendly_seats = seats;
                }
                if affected_includes(&affected, src.id, target) {
                    return true;
                }
            }
        }
        false
    }

    /// CR 603.4 evaluation context for a self-ETB trigger's condition: the
    /// entering permanent's own cast flags (kicked, bargained, X, …), so
    /// "if it was kicked" reads the cast that produced it.
    fn etb_filter_context(
        &self,
        card_id: CardId,
        controller: usize,
    ) -> crate::game::effects::EffectContext {
        let mut ctx =
            crate::game::effects::EffectContext::for_ability(card_id, controller, None);
        if let Some(c) = self.battlefield_find(card_id) {
            ctx.x_value = c.cast_x_value;
            ctx.kicked = c.kicked;
            ctx.kicked_options = c.kicked_options.clone();
            ctx.kick_count = c.kick_count;
            ctx.bargained = c.bargained;
            ctx.cast_from_hand = c.cast_from_hand;
            ctx.mana_spent_by_color = c.cast_mana_spent_by_color.clone();
        }
        ctx
    }

    pub fn fire_self_etb_triggers(&mut self, card_id: CardId, controller: usize) {
        // CR 614 — "enters under the control of an opponent of your choice"
        // (Captive Audience). A control-setting entry replacement, so it lands
        // before enters-tapped and before any ETB trigger sees a controller.
        self.apply_enters_under_opponent_control(card_id);
        // CR 614.13 — apply enters-tapped replacements before ETB triggers fire.
        self.apply_enters_tapped_replacement(card_id);
        // CR 702.179 — a "Start your engines!" permanent entering gives its
        // controller speed 1 if they have none.
        if controller < self.players.len()
            && self.players[controller].speed == 0
            && self
                .battlefield
                .iter()
                .any(|c| c.id == card_id && c.definition.keywords.contains(&crate::card::Keyword::StartYourEngines))
        {
            self.players[controller].speed = 1;
        }
        use crate::effect::{EventKind, EventScope};
        #[allow(clippy::type_complexity)]
        let etb_triggers: Vec<(Effect, Option<crate::effect::Predicate>)> = self
            .battlefield
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| {
                // Printed + statics-granted ETBs ("Slivers you control have
                // 'When this enters…'" — Lavabelly) fire alike.
                let static_granted = self.statics_granted_triggers_for(c);
                c.definition
                    .triggered_abilities
                    .iter()
                    .chain(static_granted.iter())
                    .filter(|t| t.event.kind == EventKind::EntersBattlefield
                        && matches!(t.event.scope, EventScope::SelfSource))
                    .map(|t| (t.effect.clone(), t.event.filter.clone()))
                    .collect()
            })
            .unwrap_or_default();
        // Elesh Norn replacement: zero or more copies depending on which
        // side controls a Mother of Machines. Katara, the Fearless adds an
        // extra fire for a self-source Ally ETB trigger (unless suppressed).
        let etb_mult = etb_trigger_multiplier(self, controller, Some(card_id));
        let multiplier = if etb_mult == 0 {
            0
        } else {
            etb_mult + ally_trigger_extra_fires(self, controller, card_id)
        };
        // CR 601.2b — an ETB triggered ability reads the cast's X (stamped on
        // the permanent at resolution) so filters like `ManaValueAtMostXFromCost`
        // evaluate against the real X rather than 0 (Dune Drifter).
        let cast_x = self
            .battlefield
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| c.cast_x_value)
            .unwrap_or(0);
        for (effect, filter) in etb_triggers {
            // CR 603.4 — the trigger's own condition ("When this enters, if it
            // was kicked / if delirium …") is read at fire time against the
            // entering permanent's cast flags.
            if let Some(predicate) = filter {
                let ctx = self.etb_filter_context(card_id, controller);
                if !self.evaluate_predicate(&predicate, &ctx) {
                    continue;
                }
            }
            // Strict Proctor's CR 614 replacement: pay {2} or sacrifice
            // the source. Applied once per fire of the trigger.
            if !apply_etb_trigger_tax(self, card_id, controller) {
                // Source was sacrificed; remaining fires are moot.
                return;
            }
            let auto_target = self.auto_target_for_effect_avoiding_set_x(
                &effect,
                controller,
                &[card_id],
                cast_x,
            );
            // CR 115.1c — maximize an "up to N target" self-source ETB trigger
            // (Gavony Silversmith) by filling slots 1.. with distinct picks.
            let additional =
                self.auto_extra_targets_for(&effect, card_id, controller, auto_target.clone());
            // CR 700.2b — modal ETB trigger mode pick at push-time.
            let mode = self.pick_trigger_mode(&effect, card_id, controller);
            for _ in 0..multiplier {
                self.stack.push(
                    TriggerPush::new(card_id, controller, effect.clone())
                        .target(auto_target.clone())
                        .additional_targets(additional.clone())
                        .mode(mode)
                        .x_value(cast_x)
                        .build(),
                );
            }
            // CR 603 — a triggered ability choosing targets fires
            // "becomes the target" listeners (Tenured Concocter), same as
            // the cast/activated paths. Only for effects that DECLARE a
            // target slot (printed "target …" wording), and only for
            // battlefield permanents.
            if multiplier > 0 && effect.requires_target() {
                let mut became =
                    vec![GameEvent::ChoseTargets { chooser: controller, object: card_id }];
                became.extend(auto_target.iter().chain(additional.iter()).filter_map(|t| {
                    match t {
                        Target::Permanent(id) if self.battlefield_find(*id).is_some() => {
                            Some(GameEvent::BecameTarget { target: *id, caster: controller, by: Some(card_id) })
                        }
                        _ => None,
                    }
                }));
                self.dispatch_triggers_for_events(&became);
            }
        }
    }

    // ── Cast spell ────────────────────────────────────────────────────────────

    /// Cast a modal-double-faced card via its back face. Mirrors
    /// `play_land_with_face` but for non-land back faces (creature /
    /// instant / sorcery). The card's `definition` is swapped to the back
    /// face's definition before payment + cast, so cost / type / effect
    /// all resolve against the back face. Errors with `NotALand` (reused
    /// for "no back face") if the front has no `back_face`.
    pub(crate) fn cast_spell_back_face(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.players[p].has_in_hand(card_id) {
            // MDFC back-face cast from the graveyard (Pestilent Cauldron):
            // when the card is in the controller's graveyard with the one-shot
            // `may_cast_back_from_graveyard` permission, hop it into hand for
            // the normal back-face cast pipeline (the Muldrotha idiom),
            // consuming the permission. Restore it to the graveyard on failure.
            let gy_pos = self.players[p].graveyard.iter().position(|c| {
                c.id == card_id
                    && c.may_cast_back_from_graveyard
                    && c.definition.back_face.is_some()
            });
            if gy_pos.is_some()
                && let Some(mut card) = Self::take_card(&mut self.players[p].graveyard, card_id)
            {
                card.may_cast_back_from_graveyard = false; // one-shot
                self.players[p].hand.push(card);
                let r = self.cast_spell_back_face(card_id, target, additional_targets, mode, x_value);
                if r.is_err()
                    && let Some(card) = Self::take_card(&mut self.players[p].hand, card_id)
                {
                    self.players[p].send_to_graveyard(card);
                }
                return r;
            }
            return Err(GameError::CardNotInHand(card_id));
        }
        // Snapshot the front-face definition AND look up the back face.
        // On a rejected cast the inner path returns the card to the hand
        // with its (then-swapped) back-face definition; we restore the
        // front face here so the player can still cast either face on
        // retry. Without the restore, one failed back-face cast burns the
        // front face for the rest of the game.
        let (front_def, back_def) = {
            let card = self
                .players[p]
                .hand
                .iter()
                .find(|c| c.id == card_id)
                .expect("has_in_hand verified");
            let back = match card.definition.back_face.clone() {
                Some(b) => *b,
                None => return Err(GameError::NotALand(card_id)),
            };
            (card.definition.clone(), back)
        };
        // Swap the in-hand definition to the back face in place. The
        // hand card's back_face slot is kept (it points at the back
        // we just installed), so a later restore can flip back without
        // a second catalog lookup.
        if let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id) {
            c.definition = std::sync::Arc::new(back_def);
        }
        // Delegate to the regular cast path. The back face's cost,
        // type, target filters, and effect now drive validation.
        // Tag the cast face so the SpellCast event surfaces it.
        self.pending_cast_face = CastFace::Back;
        let result = self.cast_spell(card_id, target, additional_targets, mode, x_value);
        self.pending_cast_face = CastFace::Front;
        // On rejection, the inner cast pushed the card (with the
        // back-face definition) back into the hand. Restore the front
        // face so the player can retry either face.
        if result.is_err()
            && let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id)
        {
            c.definition = front_def;
        }
        result
    }

    /// CR 702.160 — cast a Prototype artifact creature for its prototype
    /// cost. Models the alternative cast like the MDFC back face: swap the
    /// in-hand definition to the prototype-applied one (smaller cost, color,
    /// and size; same abilities/types) and flag the instance so it persists
    /// through the stack onto the battlefield and round-trips a snapshot.
    /// The regular cast pipeline then handles payment, timing, and triggers.
    pub(crate) fn cast_prototype(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let (front_def, proto_def) = {
            let card = self.players[p]
                .hand
                .iter()
                .find(|c| c.id == card_id)
                .expect("has_in_hand verified");
            match card.definition.with_prototype_applied() {
                Some(d) => (card.definition.clone(), d),
                None => return Err(GameError::InvalidTarget),
            }
        };
        if let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id) {
            c.prototype_printed = Some(front_def.clone());
            c.definition = std::sync::Arc::new(proto_def);
            c.cast_as_prototype = true;
        }
        let result = self.cast_spell(card_id, target, additional_targets, mode, x_value);
        // On rejection the inner cast returned the card to hand with the
        // prototype definition; restore the printed front face so the player
        // can still cast either face on retry.
        if result.is_err()
            && let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id)
        {
            c.definition = front_def;
            c.cast_as_prototype = false;
        }
        result
    }

    /// CR 702.140 — cast a creature with Mutate for its mutate cost, merging
    /// it onto `host` (a non-Human creature you own). Paid like a normal
    /// creature spell whose cost is the mutate cost; on resolution the spell
    /// merges instead of entering (`resolve_top_of_stack`).
    pub(crate) fn cast_mutate(
        &mut self,
        card_id: CardId,
        host: CardId,
        on_top: bool,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::CreatureType;
        let p = self.priority.player_with_priority;
        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let (printed, mutate_cost) = {
            let card = self.players[p]
                .hand
                .iter()
                .find(|c| c.id == card_id)
                .expect("has_in_hand verified");
            match card.definition.mutate.clone() {
                Some(c) => (card.definition.clone(), c),
                None => return Err(GameError::InvalidTarget),
            }
        };
        // CR 702.140a — target must be a non-Human creature its controller owns.
        let legal_host = self.battlefield.iter().any(|c| {
            c.id == host
                && c.owner == p
                && c.definition.is_creature()
                && !c.definition.has_creature_type(CreatureType::Human)
        });
        if !legal_host {
            return Err(GameError::InvalidTarget);
        }
        // Pay the mutate cost via the normal pipeline by swapping in a
        // mutate-cost definition; restore the printed face after the cast so
        // the merged pile carries the real characteristics.
        if let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id) {
            let mut d = (*printed).clone();
            d.cost = mutate_cost;
            c.definition = std::sync::Arc::new(d);
            c.mutate_onto = Some((host, on_top));
        }
        let result = self.cast_spell(card_id, None, Vec::new(), None, x_value);
        match result {
            Ok(events) => {
                // Restore the printed definition on the stack spell so the
                // merge unions the real card (its printed cost/abilities).
                for item in self.stack.iter_mut().rev() {
                    if let StackItem::Spell { card, .. } = item
                        && card.id == card_id
                    {
                        card.definition = printed;
                        break;
                    }
                }
                Ok(events)
            }
            Err(e) => {
                if let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id) {
                    c.definition = printed;
                    c.mutate_onto = None;
                }
                Err(e)
            }
        }
    }

    /// SOS Prepare — cast a copy of a prepared creature's inset prepare
    /// spell. The copy is paid for and timing-checked like a normal spell
    /// of its card type via the regular `cast_spell` pipeline (a fresh
    /// `CardInstance` is hopped through the caster's hand the way the
    /// Muldrotha / library-top paths do), then flagged `is_token` on the
    /// stack so it ceases to exist when it leaves the stack (CR 707.10a —
    /// it never hits a graveyard). Casting it removes the creature's
    /// Prepared counter ("unprepares it").
    ///
    /// Only the prepared creature's *current controller* may cast the
    /// copy — a stolen prepared creature brings its spell along.
    pub(crate) fn cast_prepare_spell(
        &mut self,
        creature_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::CounterType;
        let p = self.priority.player_with_priority;
        let Some(creature) = self.battlefield.iter().find(|c| c.id == creature_id) else {
            return Err(GameError::CardNotOnBattlefield(creature_id));
        };
        let prep_def = match creature.definition.prepare_spell.as_deref() {
            Some(d) if creature.controller == p
                && creature.counter_count(CounterType::Prepared) > 0 => d.clone(),
            _ => return Err(GameError::NotPrepared(creature_id)),
        };
        // Materialize the copy and run it through the normal cast path —
        // payment, timing (instant vs sorcery speed), target validation,
        // and cast triggers all apply to the copy's own characteristics.
        let copy_id = self.next_id();
        let copy = crate::card::CardInstance::new(copy_id, prep_def, p);
        self.players[p].hand.push(copy);
        // Register the copy before entering the cast pipeline — the cast may
        // suspend mid-way (float-spend confirm, additional-cost pick) and
        // resume via a plain `CastSpell` replay, so the bookkeeping (token
        // flag + unprepare) is settled by `settle_prepare_after_cast` wherever
        // the cast actually completes, not here.
        self.pending_prepare_copies.push((copy_id, creature_id));
        let result = self.cast_spell(copy_id, target, additional_targets, mode, x_value);
        self.settle_prepare_after_cast(copy_id, result)
    }

    /// SOS Prepare — settle the bookkeeping for a prepare-spell copy after a
    /// cast attempt (direct or a mid-cast-suspension resume replay). No-op
    /// unless `copy_id` is registered in `pending_prepare_copies`.
    ///
    /// - Copy on the stack → flag it `is_token` (CR 707.10a — it ceases to
    ///   exist off the stack; flagged only now because an in-hand token
    ///   would be swept by the token SBA), unprepare the source creature,
    ///   drop the registration.
    /// - Cast suspended again (`Ok` with a fresh `pending_decision`, copy
    ///   parked back in hand) → keep the registration for the next resume.
    /// - Cast failed (or the copy vanished) → unmaterialize the copy.
    pub(crate) fn settle_prepare_after_cast(
        &mut self,
        copy_id: CardId,
        result: Result<Vec<GameEvent>, GameError>,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::CounterType;
        let Some(idx) = self.pending_prepare_copies.iter().position(|(c, _)| *c == copy_id)
        else {
            return result;
        };
        if result.is_ok() {
            let on_stack = self.stack.iter_mut().rev().find_map(|item| match item {
                StackItem::Spell { card, .. } if card.id == copy_id => Some(card),
                _ => None,
            });
            if let Some(card) = on_stack {
                card.is_token = true;
                let (_, creature_id) = self.pending_prepare_copies.remove(idx);
                let mut events = result.unwrap_or_default();
                // Casting the copy unprepares the creature.
                if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == creature_id)
                    && c.remove_counters(CounterType::Prepared, 1) > 0
                {
                    events.push(GameEvent::CounterRemoved {
                        card_id: creature_id,
                        counter_type: CounterType::Prepared,
                        count: 1,
                    });
                }
                return Ok(events);
            }
            if self.pending_decision.is_some() {
                // Mid-cast suspension — the copy is parked in the caster's
                // hand awaiting the answer; settle again on the resume.
                return result;
            }
        }
        // The cast failed (rejected casts push the copy back into the hand)
        // — unmaterialize it.
        self.pending_prepare_copies.remove(idx);
        for player in &mut self.players {
            player.hand.retain(|c| c.id != copy_id);
        }
        result
    }

    pub fn cast_spell(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        // CR 601.3e — a card with no mana cost can't be cast by paying it
        // (suspend / free-cast paths bypass this method).
        // CR 300.2a — a card that's a land and another type (artifact lands,
        // Dryad Arbor) can only be *played*, never cast.
        {
            let p = self.priority.player_with_priority;
            if let Some(c) = self.players[p].hand.iter().find(|c| c.id == card_id) {
                if c.definition.no_mana_cost {
                    return Err(GameError::NoManaCost);
                }
                if c.definition.is_land() {
                    return Err(GameError::CannotCastLand);
                }
            }
        }
        // {X} cast costs: a hand-paying caster who didn't send an X picks
        // one via a `ChooseAmount` modal (suspend + clean replay — nothing
        // has been paid yet). Without this the client's `x_value: None`
        // resolved every human-cast X spell at X=0 (a 0/0 Fractal Summoning
        // token, a 0-card Mind Twist, …).
        //
        // Gated on [`manual_mana`], not `wants_ui`. Choosing X *is* paying a
        // cost, so it belongs to the same population as hand-picking lands —
        // and the two are not the same set, because bot seats set `wants_ui`
        // to get their decisions surfaced. The old gate deadlocked bot games
        // outright: the suspend returns `Ok`, so `would_accept` reported an
        // unaffordable X spell as castable; the bot committed it, answered
        // the modal, and the replayed cast then failed on mana or on timing.
        // `perform_action`'s rollback restored the pending decision along
        // with everything else, so the bot answered the same decision the
        // same way forever. That livelocked ~14 % of cube games, and they
        // were dropped from every ladder and recommender measurement.
        {
            let p = self.priority.player_with_priority;
            if x_value.is_none()
                && self.players[p].manual_mana
                && let Some(card) = self.find_card_anywhere(card_id)
                && (card.definition.cost.has_x() || card.definition.additional_cost_pay_x_life)
            {
                // A "pay X life" additional cost with no {X} mana pip is
                // bounded by the caster's life total (CR 119.4), not mana.
                let max = if card.definition.additional_cost_pay_x_life
                    && !card.definition.cost.has_x()
                {
                    self.effective_life(p).max(0) as u32
                } else {
                    self.max_prompt_x(p, &card.definition.cost)
                };
                let source_name = card.definition.name.to_string();
                self.pending_decision = Some(crate::game::types::PendingDecision {
                    decision: crate::decision::Decision::ChooseAmount {
                        source: card_id,
                        max,
                        prompt: format!("{source_name}: choose X"),
                    },
                    resume: crate::game::types::ResumeContext::CastXPick {
                        caster: p,
                        action: Box::new(crate::game::types::GameAction::CastSpell {
                            card_id,
                            target,
                            additional_targets,
                            mode,
                            x_value: None,
                        }),
                    },
                });
                return Ok(vec![]);
            }
        }
        // Slot-0 targets that live in an off-board zone ("return target
        // nonland card from your graveyard …"): the client's targeting
        // cursor can't select graveyard/exile cards, so it submits the cast
        // with no target — gather the pick here via a `ChooseCards` modal.
        {
            let p = self.priority.player_with_priority;
            let slot0 = if target.is_none() && self.players[p].wants_ui {
                self.find_card_anywhere(card_id).and_then(|card| {
                    card.definition
                        .effect
                        .target_filter_for_slot_in_mode(0, mode)
                        .filter(|f| f.mentions_offboard_zone())
                        .map(|f| {
                            (
                                f.resolve_x(x_value.unwrap_or(0)),
                                card.definition.name.to_string(),
                            )
                        })
                })
            } else {
                None
            };
            if let Some((filter, source_name)) = slot0 {
                let candidates: Vec<(CardId, String)> = self
                    .players
                    .iter()
                    .flat_map(|pl| pl.graveyard.iter())
                    .chain(self.exile.iter())
                    .filter(|c| {
                        c.id != card_id
                            && self.evaluate_requirement_static(
                                &filter,
                                &Target::Permanent(c.id),
                                p,
                                Some(card_id),
                            )
                    })
                    .map(|c| (c.id, c.definition.name.to_string()))
                    .collect();
                if candidates.is_empty() {
                    return Err(GameError::SelectionRequirementViolated);
                }
                self.pending_decision = Some(crate::game::types::PendingDecision {
                    decision: crate::decision::Decision::ChooseCards {
                        source: card_id,
                        prompt: format!("{source_name}: choose a card to target"),
                        candidates,
                        min: 1,
                        max: 1,
                    },
                    resume: crate::game::types::ResumeContext::CastSlot0TargetPick {
                        caster: p,
                        action: Box::new(crate::game::types::GameAction::CastSpell {
                            card_id,
                            target: None,
                            additional_targets,
                            mode,
                            x_value,
                        }),
                    },
                });
                return Ok(vec![]);
            }
        }
        // Multi-target spells (Chelonian Tackle's "then it fights up to one
        // target creature an opponent controls"): the client's cast flow only
        // collects slot 0, so slots 1+ arrive empty and the extra half of the
        // effect silently no-ops. Bind the next missing slot here — a
        // `wants_ui` caster picks via a `ChooseTarget` cursor decision
        // (suspend + clean replay; nothing has been paid yet), everyone else
        // auto-fills. No legal candidate leaves the slot empty, which is the
        // printed "up to one" behavior.
        {
            let p = self.priority.player_with_priority;
            // A `DeclineTarget` answer replays the cast with this card id
            // stamped in the suppress scratch — consume it and skip the
            // prompt so declining ends target selection.
            let suppressed = if self.suppress_extra_target_prompts == Some(card_id) {
                self.suppress_extra_target_prompts = None;
                true
            } else {
                false
            };
            let slot_info = if !suppressed && target.is_some() && self.players[p].wants_ui {
                self.find_card_anywhere(card_id).and_then(|card| {
                    // "Extra target only on your main phase" spells (Return
                    // to Dust) genuinely cast single-target off-main — don't
                    // prompt for the slot the rules forbid.
                    if card.definition.extra_targets_main_phase_only
                        && !(self.active_player_idx == p && self.step.is_main_phase())
                    {
                        return None;
                    }
                    let slot = 1 + additional_targets.len() as u8;
                    card.definition
                        .effect
                        .target_filter_for_slot_in_mode(slot, mode)
                        .map(|f| {
                            (
                                f.resolve_x(x_value.unwrap_or(0)),
                                card.definition.name.to_string(),
                                // "Up to N targets" slots past the printed
                                // minimum may be declined.
                                card.definition.effect.target_slot_optional_x(
                                    slot,
                                    mode,
                                    x_value.unwrap_or(0),
                                ),
                                // CR 601.4d — the slots of one multi-target
                                // instance must name distinct objects.
                                card.definition
                                    .effect
                                    .distinct_target_count(mode)
                                    .is_some_and(|n| slot < n),
                            )
                        })
                })
            } else {
                None
            };
            if let Some((filter, source_name, optional, distinct)) = slot_info {
                let chosen: Vec<&Target> =
                    target.iter().chain(additional_targets.iter()).collect();
                let candidates: Vec<Target> = self
                    .battlefield
                    .iter()
                    .map(|c| Target::Permanent(c.id))
                    .chain((0..self.players.len()).map(Target::Player))
                    .filter(|t| {
                        (!distinct || !chosen.contains(&t))
                            && self.evaluate_requirement_static(&filter, t, p, Some(card_id))
                            && self.check_target_legality(t, p).is_ok()
                    })
                    .collect();
                if !candidates.is_empty() {
                    self.pending_decision = Some(crate::game::types::PendingDecision {
                        decision: crate::decision::Decision::ChooseTarget {
                            optional,
                            source: card_id,
                            legal: candidates,
                            source_name,
                            description: "choose an additional target".into(),
                        },
                        resume: crate::game::types::ResumeContext::CastExtraTargetPick {
                            caster: p,
                            action: Box::new(crate::game::types::GameAction::CastSpell {
                                card_id,
                                target,
                                additional_targets,
                                mode,
                                x_value,
                            }),
                        },
                    });
                    return Ok(vec![]);
                }
            }
        }
        // Muldrotha — cast a permanent spell of each permanent type from
        // your graveyard during each of your turns. Hop the card into hand
        // for the normal cast pipeline; restore on failure, record the
        // consumed permanent type on success.
        let p = self.priority.player_with_priority;
        if !self.players[p].hand.iter().any(|c| c.id == card_id)
            && !self.players[p].graveyard.iter().any(|c| {
                c.id == card_id
                    && self.cast_from_zone_blocked(p, &c.definition, crate::card::Zone::Graveyard)
            })
            && let Some(used_type) = self.graveyard_cast_type_available(p, card_id)
        {
            let card = Self::take_card(&mut self.players[p].graveyard, card_id)
                .ok_or(GameError::CardNotInHand(card_id))?;
            self.players[p].hand.push(card);
            let r = self.cast_spell_with_convoke(
                card_id, target, additional_targets, mode, x_value, &[], &[], CastFlags::default(),
            );
            match &r {
                Err(_) => {
                    if let Some(card) = Self::take_card(&mut self.players[p].hand, card_id) {
                        self.players[p].send_to_graveyard(card);
                    }
                }
                Ok(_) => {
                    if let Some(t) = used_type {
                        self.players[p].graveyard_cast_types_this_turn.push(t);
                    }
                    self.entered_from_graveyard_this_turn.insert(card_id);
                }
            }
            return r;
        }
        // Noctis — cast a covered spell from your graveyard by paying the
        // static's life surcharge; it enters with a finality counter. Hop the
        // card into hand for the normal cast pipeline; restore on failure.
        if !self.players[p].hand.iter().any(|c| c.id == card_id)
            && !self.players[p].graveyard.iter().any(|c| {
                c.id == card_id
                    && self.cast_from_zone_blocked(p, &c.definition, crate::card::Zone::Graveyard)
            })
            && let Some(life) = self.graveyard_cast_life_surcharge(p, card_id)
        {
            let mut card = Self::take_card(&mut self.players[p].graveyard, card_id)
                .ok_or(GameError::CardNotInHand(card_id))?;
            card.pending_etb_counters.push((crate::card::CounterType::Finality, 1));
            self.players[p].hand.push(card);
            let r = self.cast_spell_with_convoke(
                card_id, target, additional_targets, mode, x_value, &[], &[], CastFlags::default(),
            );
            match &r {
                Err(_) => {
                    if let Some(mut card) = Self::take_card(&mut self.players[p].hand, card_id) {
                        card.pending_etb_counters.clear();
                        self.players[p].send_to_graveyard(card);
                    }
                }
                Ok(_) => {
                    self.pay_life_cost(p, life);
                    self.entered_from_graveyard_this_turn.insert(card_id);
                }
            }
            return r;
        }
        // Osteomancer Adept — cast a creature spell from your graveyard by
        // foraging in addition to its other costs; it enters with a finality
        // counter. Same hop-into-hand shape as the Noctis branch above.
        if !self.players[p].hand.iter().any(|c| c.id == card_id)
            && self.players[p].forage_graveyard_casts_turn == Some(self.turn_number)
            && self.can_forage(p)
            && self.players[p].graveyard.iter().any(|c| {
                c.id == card_id
                    && c.definition.is_creature()
                    && !self.cast_from_zone_blocked(p, &c.definition, crate::card::Zone::Graveyard)
            })
        {
            let mut card = Self::take_card(&mut self.players[p].graveyard, card_id)
                .ok_or(GameError::CardNotInHand(card_id))?;
            card.pending_etb_counters.push((crate::card::CounterType::Finality, 1));
            self.players[p].hand.push(card);
            let r = self.cast_spell_with_convoke(
                card_id, target, additional_targets, mode, x_value, &[], &[], CastFlags::default(),
            );
            match r {
                Err(e) => {
                    if let Some(mut card) = Self::take_card(&mut self.players[p].hand, card_id) {
                        card.pending_etb_counters.clear();
                        self.players[p].send_to_graveyard(card);
                    }
                    return Err(e);
                }
                Ok(mut evs) => {
                    evs.append(&mut self.pay_forage(p));
                    self.entered_from_graveyard_this_turn.insert(card_id);
                    return Ok(evs);
                }
            }
        }
        // Bolas's Citadel — cast a spell off the library top paying life equal
        // to its mana value instead of its mana cost. Hop to hand, pay life,
        // and free-cast; restore the card to the top on failure.
        if !self.players[p].hand.iter().any(|c| c.id == card_id)
            && !self.players[p].library.first().is_some_and(|c| {
                self.cast_from_zone_blocked(p, &c.definition, crate::card::Zone::Library)
            })
            && let Some(life) = self.library_top_pay_life_cost(p, card_id)
        {
            if self.players[p].life < life as i32 {
                return Err(GameError::InsufficientLife);
            }
            let card = self.players[p].library.remove(0);
            self.players[p].hand.push(card);
            let r = self.cast_card_for_free(
                p, card_id, crate::card::Zone::Hand, target, additional_targets.clone(), mode, x_value, false,
            );
            match r {
                Err(e) => {
                    if let Some(card) = Self::take_card(&mut self.players[p].hand, card_id) {
                        self.players[p].library.insert(0, card);
                    }
                    return Err(e);
                }
                Ok(events) => {
                    self.pay_life_cost(p, life);
                    return Ok(events);
                }
            }
        }
        // CR 401.6 — cast off the library top when a PlayFromLibraryTop
        // static covers the card (Mystic Forge). Hop the card into hand for
        // the normal cast pipeline; restore it to the top on failure.
        if !self.players[p].hand.iter().any(|c| c.id == card_id)
            && !self.players[p].library.first().is_some_and(|c| {
                self.cast_from_zone_blocked(p, &c.definition, crate::card::Zone::Library)
            })
            && self.library_top_playable(p, card_id)
        {
            let capped = self.library_top_cast_is_capped(p, card_id);
            let card = self.players[p].library.remove(0);
            self.players[p].hand.push(card);
            // The cast pipeline runs from hand, so record the true origin for
            // "cast a spell from your library" payoffs (Melek).
            self.casting_from_library_top = Some(card_id);
            let r = self.cast_spell_with_convoke(
                card_id, target, additional_targets, mode, x_value, &[], &[], CastFlags::default(),
            );
            self.casting_from_library_top = None;
            if r.is_err() {
                if let Some(card) = Self::take_card(&mut self.players[p].hand, card_id) {
                    self.players[p].library.insert(0, card);
                }
            } else if capped {
                self.players[p].cast_from_library_top_this_turn = true;
            }
            return r;
        }
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[], &[], CastFlags::default())
    }

    /// When `card_id` sits in `p`'s graveyard on `p`'s turn and a
    /// graveyard-cast permission is active, say whether it can be cast:
    /// `Some(None)` for an unlimited permission (Hades, Sorcerer of Eld —
    /// nothing is consumed), `Some(Some(type))` for Muldrotha's one-per-
    /// permanent-type budget, `None` when no permission covers it.
    pub(crate) fn graveyard_cast_type_available(
        &self,
        p: usize,
        card_id: CardId,
    ) -> Option<Option<crate::card::CardType>> {
        use crate::card::CardType;
        use crate::effect::StaticEffect;
        if self.active_player_idx != p {
            return None;
        }
        let card = self.players[p].graveyard.iter().find(|c| c.id == card_id)?;
        if self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::PlayCardsFromGraveyardDuringYourTurn)
                })
        }) {
            return Some(None);
        }
        let permission = self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::MayCastPermanentsFromGraveyard)
                })
        });
        if !permission {
            return None;
        }
        const PERMANENT_TYPES: [CardType; 5] = [
            CardType::Artifact,
            CardType::Creature,
            CardType::Enchantment,
            CardType::Planeswalker,
            CardType::Battle,
        ];
        card.definition
            .card_types
            .iter()
            .find(|t| {
                PERMANENT_TYPES.contains(t)
                    && !self.players[p].graveyard_cast_types_this_turn.contains(t)
            })
            .cloned()
            .map(Some)
    }

    /// CR 701.61 — pay the forage cost for `p`: exile three graveyard cards,
    /// or sacrifice a Food when the graveyard is too small. Emits `Foraged`.
    /// Shared by the `ForageOrPay` additional cost and Osteomancer Adept's
    /// graveyard-cast grant.
    pub(crate) fn pay_forage(&mut self, p: usize) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let gy_ids: Vec<CardId> =
            self.players[p].graveyard.iter().take(3).map(|c| c.id).collect();
        if gy_ids.len() >= 3 {
            let ctx = EffectContext::for_spell(p, None, 0, 0);
            for id in gy_ids {
                self.move_card_to(id, &crate::effect::ZoneDest::Exile, &ctx, &mut events);
            }
        } else if let Some(fid) = self
            .battlefield
            .iter()
            .find(|c| {
                c.controller == p
                    && c.definition
                        .subtypes
                        .artifact_subtypes
                        .contains(&crate::card::ArtifactSubtype::Food)
            })
            .map(|c| c.id)
        {
            events.push(GameEvent::PermanentSacrificed { card_id: fid, who: p });
            let mut die = self.remove_to_graveyard_with_triggers(fid);
            events.append(&mut die);
        }
        events.push(GameEvent::Foraged { player: p });
        events
    }

    /// Noctis — when `card_id` sits in `p`'s graveyard, a
    /// `GraveyardCastWithLifeSurcharge` permission `p` controls covers it, and
    /// `p` can pay the life (CR 119.4), return the surcharge.
    pub(crate) fn graveyard_cast_life_surcharge(&self, p: usize, card_id: CardId) -> Option<u32> {
        use crate::effect::StaticEffect;
        let card = self.players[p].graveyard.iter().find(|c| c.id == card_id)?;
        // The permission can come from a permanent you control (Noctis) or from
        // the graveyard card's own static (Hundred-Battle Veteran — "you may
        // cast this card from your graveyard"), so include the card itself.
        self.battlefield
            .iter()
            .filter(|c| c.controller == p)
            .flat_map(|c| c.definition.static_abilities.iter())
            .chain(card.definition.static_abilities.iter())
            .find_map(|sa| match &sa.effect {
                StaticEffect::GraveyardCastWithLifeSurcharge { filter, life }
                    if self.evaluate_requirement_on_card(filter, card, p)
                        && self.players[p].life >= *life as i32 =>
                {
                    Some(*life)
                }
                _ => None,
            })
    }

    /// CR 401.6 — true when `card_id` is the top card of `p`'s library and a
    /// `PlayFromLibraryTop` static `p` controls covers it.
    pub fn library_top_playable(&self, p: usize, card_id: CardId) -> bool {
        use crate::effect::StaticEffect;
        let Some(card) = self.players[p].library.first() else { return false };
        if card.id != card_id {
            return false;
        }
        // CR 401.6 — turn-scoped "play lands and cast spells from the top of
        // your library" grant (The Belligerent) covers any top card.
        if self.players[p].play_from_top_this_turn {
            return true;
        }
        let capped_used = self.players[p].cast_from_library_top_this_turn;
        self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition.static_abilities.iter().any(|sa| match &sa.effect {
                    StaticEffect::PlayFromLibraryTop { filter }
                    | StaticEffect::PlayFromLibraryTopPayLife { filter } => {
                        self.evaluate_requirement_on_card(filter, card, p)
                    }
                    // Johann — the once-per-turn grant lapses after the first
                    // top-of-library cast this turn.
                    StaticEffect::PlayFromLibraryTopOncePerTurn { filter } => {
                        !capped_used && self.evaluate_requirement_on_card(filter, card, p)
                    }
                    _ => false,
                })
        })
    }

    /// Bolas's Citadel — if `card_id` is the top card of `p`'s library, covered
    /// by a `PlayFromLibraryTopPayLife` static, and is a spell (not a land),
    /// return the life to pay in lieu of its mana cost (its mana value).
    pub(crate) fn library_top_pay_life_cost(&self, p: usize, card_id: crate::card::CardId) -> Option<u32> {
        use crate::effect::StaticEffect;
        let card = self.players[p].library.first()?;
        if card.id != card_id || card.definition.is_land() {
            return None;
        }
        let covered = self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition.static_abilities.iter().any(|sa| matches!(
                    &sa.effect,
                    StaticEffect::PlayFromLibraryTopPayLife { filter }
                        if self.evaluate_requirement_on_card(filter, card, p)
                ))
        });
        covered.then(|| card.definition.cost.cmc())
    }

    /// True when the *only* grant letting `p` play `card_id` off the library
    /// top is a `PlayFromLibraryTopOncePerTurn` (Johann) — so casting it should
    /// consume the once-per-turn charge. If any uncapped grant also covers the
    /// card, the charge is not spent.
    pub(crate) fn library_top_cast_is_capped(&self, p: usize, card_id: CardId) -> bool {
        use crate::effect::StaticEffect;
        if self.players[p].play_from_top_this_turn {
            return false;
        }
        let Some(card) = self.players[p].library.first() else { return false };
        if card.id != card_id {
            return false;
        }
        let mut capped = false;
        for c in self.battlefield.iter().filter(|c| c.controller == p) {
            for sa in &c.definition.static_abilities {
                match &sa.effect {
                    StaticEffect::PlayFromLibraryTop { filter }
                        if self.evaluate_requirement_on_card(filter, card, p) =>
                    {
                        return false;
                    }
                    StaticEffect::PlayFromLibraryTopOncePerTurn { filter }
                        if self.evaluate_requirement_on_card(filter, card, p) =>
                    {
                        capped = true;
                    }
                    _ => {}
                }
            }
        }
        capped
    }

    /// CR 702.32 — cast a spell paying its optional Kicker cost. The kicker
    /// mana is added to the spell's cost and the resolving spell is stamped
    /// `kicked`, which `Predicate::SpellWasKicked` reads.
    /// CR 702.32b — cast a spell paying the chosen subset of its
    /// `kicker_options`. The picks are stamped onto the spell so
    /// `Predicate::SpellWasKickedWith` riders fire per option (the Volvers).
    pub(crate) fn cast_spell_kickers(
        &mut self,
        card_id: CardId,
        kickers: Vec<u8>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_kicker_options = kickers;
        let out = self.cast_spell_with_convoke(
            card_id,
            target,
            additional_targets,
            mode,
            x_value,
            &[],
            &[],
            CastFlags { kicked: true, ..Default::default() },
        );
        self.cast_kicker_options.clear();
        out
    }

    pub(crate) fn cast_spell_kicked(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[], &[], CastFlags { kicked: true, ..Default::default() })
    }

    /// CR 702.153 — cast a spell paying its optional Casualty cost. The named
    /// creature (controlled by the caster, power ≥ the casualty number) is
    /// sacrificed as an additional cost before the spell is put on the stack;
    /// then the just-cast spell is copied (the copy's controller may choose
    /// new targets — AutoDecider keeps the originals).
    /// CR 601.2h — run `f` on a throwaway clone first; only when the whole
    /// additional-cost cast sequence succeeds is it re-applied to the real
    /// state, so a late payment failure can't leave partial state (a spell
    /// committed at base cost, or cost sacrifices with no spell).
    fn cast_atomically(
        &mut self,
        f: impl Fn(&mut Self) -> Result<Vec<GameEvent>, GameError>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let mut probe = self.clone();
        f(&mut probe)?;
        f(self)
    }

    pub(crate) fn cast_spell_casualty(
        &mut self,
        card_id: CardId,
        sacrifice: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_atomically(|g| {
            g.cast_spell_casualty_inner(card_id, sacrifice, target.clone(), additional_targets.clone(), mode, x_value)
        })
    }

    fn cast_spell_casualty_inner(
        &mut self,
        card_id: CardId,
        sacrifice: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        // Validate the casualty number and the sacrifice creature up front so
        // a rejected cast doesn't sacrifice anything.
        let n = self
            .players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| c.definition.casualty_cost())
            .ok_or(GameError::CardNotInHand(card_id))?;
        let sac_ok = self.battlefield.iter().any(|c| {
            c.id == sacrifice
                && c.controller == p
                && c.definition.is_creature()
                && c.power().max(0) as u32 >= n
        });
        if !sac_ok {
            return Err(GameError::InvalidTarget);
        }
        // Pay the casualty cost (CR 601.2b additional cost): sacrifice now, so
        // its death triggers go on the stack under the spell.
        if let Some(c) = self.dying_snapshot(sacrifice) {
            self.died_card_snapshots.insert(sacrifice, c);
        }
        let mut events = vec![
            GameEvent::CreatureSacrificed { card_id: sacrifice, who: p },
            GameEvent::CreatureDied { card_id: sacrifice },
            GameEvent::PermanentSacrificed { card_id: sacrifice, who: p },
        ];
        let mut die = self.remove_to_graveyard_with_triggers(sacrifice);
        events.append(&mut die);
        // Cast the spell normally, then copy it on the stack (CR 702.153a).
        let mut cast_events = self.cast_spell(card_id, target, additional_targets, mode, x_value)?;
        events.append(&mut cast_events);
        self.copy_stack_spell(card_id, 1, true, &mut events);
        Ok(events)
    }

    /// CR 702.157 — cast a creature spell paying its optional Squad cost
    /// `times` times. The squad cost is charged that many extra times (an
    /// additional cost, CR 601.2f) and the resolving spell is stamped
    /// `squad_count = times` so its ETB mints that many token copies.
    pub(crate) fn cast_spell_squad(
        &mut self,
        card_id: CardId,
        times: u32,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_atomically(|g| {
            g.cast_spell_squad_inner(card_id, times, target.clone(), additional_targets.clone(), mode, x_value)
        })
    }

    fn cast_spell_squad_inner(
        &mut self,
        card_id: CardId,
        times: u32,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let squad = self
            .players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| c.definition.squad_cost().cloned())
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Pay the base cost first (pip-aware, so the colored pips aren't
        // stranded by the generic squad payment), then charge the squad cost
        // `times` times as an additional cost (CR 601.2f).
        let events = self.cast_spell(card_id, target, additional_targets, mode, x_value)?;
        if times > 0 {
            let mut combined = crate::mana::ManaCost { symbols: Vec::new() };
            for _ in 0..times {
                combined.symbols.extend(squad.symbols.iter().cloned());
            }
            self.try_pay_with_auto_tap(p, &combined)?;
        }
        // Stamp the squad count on the spell now on the stack so its ETB
        // (Value::SquadCount) mints the right number of copies.
        if times > 0 {
            for si in self.stack.iter_mut() {
                if let StackItem::Spell { card, .. } = si
                    && card.id == card_id
                {
                    card.squad_count = times;
                }
            }
        }
        Ok(events)
    }

    /// CR 702.33c — cast a spell paying its Multikicker cost `times` times.
    /// The kicker cost is charged that many extra times (CR 601.2f) and the
    /// resolving spell is stamped `kicked` + `kick_count = times` so
    /// `Value::TimesKicked` riders read it (Everflowing Chalice).
    pub(crate) fn cast_spell_multikicked(
        &mut self,
        card_id: CardId,
        times: u32,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_atomically(|g| {
            g.cast_spell_multikicked_inner(card_id, times, target.clone(), additional_targets.clone(), mode, x_value)
        })
    }

    fn cast_spell_multikicked_inner(
        &mut self,
        card_id: CardId,
        times: u32,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let kick = self
            .players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| c.definition.has_multikicker().cloned())
            .ok_or(GameError::CardNotInHand(card_id))?;
        // CR 601.2f/h — the kicker cost is paid as part of casting, so pay it
        // and stamp the count BEFORE the cast pipeline fires its spell-cast
        // triggers (Rumbling Aftershocks reads the kick count off the stack).
        if times > 0 {
            let mut combined = crate::mana::ManaCost { symbols: Vec::new() };
            for _ in 0..times {
                combined.symbols.extend(kick.symbols.iter().cloned());
            }
            self.try_pay_with_auto_tap(p, &combined)?;
        }
        self.cast_kick_count = times;
        let events = self.cast_spell(card_id, target, additional_targets, mode, x_value);
        self.cast_kick_count = 0;
        events
    }

    /// CR 702.107 — cast an instant/sorcery paying its optional Replicate cost
    /// `times` times. The replicate cost is charged that many extra times and
    /// the spell is copied that many times on the stack (copies may choose new
    /// targets).
    pub(crate) fn cast_spell_replicate(
        &mut self,
        card_id: CardId,
        times: u32,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_atomically(|g| {
            g.cast_spell_replicate_inner(card_id, times, target.clone(), additional_targets.clone(), mode, x_value)
        })
    }

    fn cast_spell_replicate_inner(
        &mut self,
        card_id: CardId,
        times: u32,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let def = self
            .players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| &c.definition)
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Djinn Illuminatus grants replicate to the caster's instants and
        // sorceries, with the spell's own mana cost as the replicate cost.
        let mana_replicate = def
            .replicate_cost()
            .cloned()
            .or_else(|| self.granted_replicate_cost(p, def));
        let energy_per = def.replicate_energy_cost();
        // Energy-paid replicate (Reiterating Bolt) must have the energy up front.
        if let Some(n) = energy_per
            && self.players[p].energy < n.saturating_mul(times)
        {
            return Err(GameError::InsufficientEnergy);
        }
        // Base cost first (pip-aware), then the replicate cost `times` times.
        let mut events = self.cast_spell(card_id, target, additional_targets, mode, x_value)?;
        if times > 0 {
            if let Some(n) = energy_per {
                self.spend_energy(p, n.saturating_mul(times));
            } else if let Some(replicate) = mana_replicate {
                let mut combined = crate::mana::ManaCost { symbols: Vec::new() };
                for _ in 0..times {
                    combined.symbols.extend(replicate.symbols.iter().cloned());
                }
                self.try_pay_with_auto_tap(p, &combined)?;
            }
            // CR 702.107a — copy the spell once per replicate payment; copies
            // may choose new targets.
            self.copy_stack_spell(card_id, times as usize, true, &mut events);
        }
        Ok(events)
    }

    /// CR 702.78 — cast a spell paying its optional Conspire cost: tap two
    /// untapped creatures you control that each share a color with the spell;
    /// the spell is then copied once (the copy may choose new targets).
    pub(crate) fn cast_spell_conspire(
        &mut self,
        card_id: CardId,
        conspire_creatures: [CardId; 2],
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_atomically(|g| {
            g.cast_spell_conspire_inner(card_id, conspire_creatures, target.clone(), additional_targets.clone(), mode, x_value)
        })
    }

    fn cast_spell_conspire_inner(
        &mut self,
        card_id: CardId,
        conspire_creatures: [CardId; 2],
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let [c0, c1] = conspire_creatures;
        if c0 == c1 {
            return Err(GameError::InvalidTarget);
        }
        // Spell must have Conspire and at least one color (a colorless spell
        // can never satisfy "shares a color" — CR 702.79b).
        let spell_colors = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .filter(|c| c.definition.keywords.contains(&crate::card::Keyword::Conspire))
            .map(|c| c.definition.printed_colors())
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Each conspirer must be an untapped creature you control sharing a
        // color with the spell (computed colors, so granted/hybrid count).
        for cid in [c0, c1] {
            let on_bf = self.battlefield.iter().any(|c| {
                c.id == cid && c.controller == p && !c.tapped && c.definition.is_creature()
            });
            let shares = self
                .computed_permanent(cid)
                .map(|cp| cp.colors.iter().any(|col| spell_colors.contains(&col)))
                .unwrap_or(false);
            if !on_bf || !shares {
                return Err(GameError::InvalidTarget);
            }
        }
        // Tap both as the additional cost, then cast and copy once.
        for cid in [c0, c1] {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) {
                c.tapped = true;
            }
        }
        let mut events = self.cast_spell(card_id, target, additional_targets, mode, x_value)?;
        self.copy_stack_spell(card_id, 1, true, &mut events);
        Ok(events)
    }

    /// CR 601.2b — cast a spell paying its optional "sacrifice any number of
    /// creatures, {N} less each" additional cost (Awaken the Blood Avatar).
    /// Each creature in `sacrifices` is sacrificed before the spell is put on
    /// the stack; the generic cost drops by the card's
    /// `sacrifice_cost_reduction` per creature, threaded through the normal
    /// cast path via the transient `extra_cast_reduction`.
    pub(crate) fn cast_spell_sacrifice_reduce(
        &mut self,
        card_id: CardId,
        sacrifices: Vec<CardId>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_atomically(|g| {
            g.cast_spell_sacrifice_reduce_inner(card_id, sacrifices.clone(), target.clone(), additional_targets.clone(), mode, x_value)
        })
    }

    fn cast_spell_sacrifice_reduce_inner(
        &mut self,
        card_id: CardId,
        sacrifices: Vec<CardId>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::effect::StaticEffect;
        let p = self.priority.player_with_priority;
        // Validate the card is in hand and carries the optional cost.
        let per = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| {
                c.definition.static_abilities.iter().find_map(|sa| match sa.effect {
                    StaticEffect::SacrificeCostReduction { per } => Some(per),
                    _ => None,
                })
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Validate every sacrifice is a distinct creature the caster controls.
        for sac in &sacrifices {
            let ok = self
                .battlefield
                .iter()
                .any(|c| c.id == *sac && c.controller == p && c.definition.is_creature());
            if !ok {
                return Err(GameError::InvalidTarget);
            }
        }
        // Sacrifice the chosen creatures as an additional cost (their death
        // triggers go on the stack under the spell).
        let mut events = Vec::new();
        for sac in &sacrifices {
            if let Some(c) = self.dying_snapshot(*sac) {
                self.died_card_snapshots.insert(*sac, c);
            }
            events.push(GameEvent::CreatureSacrificed { card_id: *sac, who: p });
            events.push(GameEvent::CreatureDied { card_id: *sac });
            events.push(GameEvent::PermanentSacrificed { card_id: *sac, who: p });
            let mut die = self.remove_to_graveyard_with_triggers(*sac);
            events.append(&mut die);
        }
        // Stamp the transient reduction, cast through the normal path, clear.
        self.extra_cast_reduction = per.saturating_mul(sacrifices.len() as u32);
        let cast = self.cast_spell(card_id, target, additional_targets, mode, x_value);
        self.extra_cast_reduction = 0;
        let mut cast_events = cast?;
        events.append(&mut cast_events);
        Ok(events)
    }

    /// CR 702.176 — cast a spell paying its optional Bargain cost. If
    /// `sacrifice` is `Some`, that artifact/enchantment/token the caster
    /// controls is sacrificed as an additional cost and the resolving spell is
    /// stamped `bargained` (read by `Predicate::SpellWasBargained`).
    /// CR 702.47 — cast `card_id` splicing the given hand cards onto it.
    /// Each splice card must carry a `Keyword::Splice` whose quality matches
    /// one of the spell's subtypes; it stays in hand (revealed), its splice
    /// cost is paid additionally, and its rules text resolves after the main
    /// effect (spliced effect `i` targets `additional_targets[i]`).
    pub(crate) fn cast_spell_spliced(
        &mut self,
        card_id: CardId,
        splice_cards: &[CardId],
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let splice_cards = splice_cards.to_vec();
        self.cast_atomically(move |g| {
            g.cast_spell_spliced_inner(
                card_id,
                &splice_cards,
                target.clone(),
                additional_targets.clone(),
                mode,
                x_value,
            )
        })
    }

    fn cast_spell_spliced_inner(
        &mut self,
        card_id: CardId,
        splice_cards: &[CardId],
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Keyword;
        let p = self.priority.player_with_priority;
        if splice_cards.is_empty() || splice_cards.contains(&card_id) {
            return Err(GameError::InvalidTarget);
        }
        let mut seen = std::collections::HashSet::new();
        let spell_subtypes = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| c.definition.subtypes.spell_subtypes.clone())
            .ok_or(GameError::CardNotInHand(card_id))?;
        let mut spliced_effects = Vec::new();
        let mut spliced_names = Vec::new();
        let mut cost_events = Vec::new();
        for &sid in splice_cards {
            // CR 702.47b — no card spliced onto the same spell twice.
            if !seen.insert(sid) {
                return Err(GameError::InvalidTarget);
            }
            let splicer = self.players[p]
                .hand
                .iter()
                .find(|c| c.id == sid)
                .ok_or(GameError::CardNotInHand(sid))?;
            let (cost, quality) = splicer
                .definition
                .keywords
                .iter()
                .find_map(|k| match k {
                    Keyword::Splice(cost, quality) => Some((cost.clone(), *quality)),
                    _ => None,
                })
                .ok_or(GameError::InvalidTarget)?;
            if !spell_subtypes.contains(&quality) {
                return Err(GameError::InvalidTarget);
            }
            let extra = splicer.definition.splice_extra_cost.clone();
            spliced_names.push(splicer.definition.name.to_string());
            spliced_effects.push(splicer.definition.effect.clone());
            // The splice cost is an additional cost (601.2b); the card stays
            // in hand. CR 702.47's non-mana half (Torrent of Stone's "sacrifice
            // two Mountains") rides the shared additional-cost payer.
            if let Some(extra) = extra {
                let costs = [extra];
                if !self.additional_costs_payable(p, &costs) {
                    return Err(GameError::InvalidTarget);
                }
                self.try_pay_with_auto_tap(p, &cost)?;
                let (mut ev, _) = self.pay_additional_costs(p, &costs, None, None);
                cost_events.append(&mut ev);
            } else {
                self.try_pay_with_auto_tap(p, &cost)?;
            }
        }
        // CR 702.47b — spliced effect `i` reads its target from
        // `additional_targets[i]`. A caller that didn't pre-pick them (the
        // client's splice picker, the bot) gets each targeting splicer
        // auto-aimed the way a plain cast would be.
        let mut additional_targets = additional_targets;
        for (i, eff) in spliced_effects.iter().enumerate() {
            if additional_targets.get(i).is_some() || !eff.requires_target() {
                continue;
            }
            additional_targets.resize(i, Target::Player(p));
            match self.auto_target_for_effect(eff, p) {
                Some(t) => additional_targets.push(t),
                // No legal target: the spliced clause resolves as a no-op
                // rather than blocking the whole cast (CR 608.2b).
                None => additional_targets.push(Target::Player(p)),
            }
        }
        if let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id) {
            c.spliced_effects = spliced_effects;
            c.spliced_names = spliced_names;
        }
        let mut events = self.cast_spell(card_id, target, additional_targets, mode, x_value)?;
        cost_events.append(&mut events);
        Ok(cost_events)
    }

    pub(crate) fn cast_spell_bargain(
        &mut self,
        card_id: CardId,
        sacrifice: Option<CardId>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_atomically(|g| {
            g.cast_spell_bargain_inner(card_id, sacrifice, target.clone(), additional_targets.clone(), mode, x_value)
        })
    }

    fn cast_spell_bargain_inner(
        &mut self,
        card_id: CardId,
        sacrifice: Option<CardId>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Keyword;
        use crate::effect::StaticEffect;
        let p = self.priority.player_with_priority;
        let has_bargain = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| c.definition.keywords.contains(&Keyword::Bargain))
            .ok_or(GameError::CardNotInHand(card_id))?;
        if !has_bargain {
            return Err(GameError::CardNotInHand(card_id));
        }
        let mut events = Vec::new();
        // Pay the optional Bargain cost: sacrifice one artifact / enchantment /
        // token the caster controls.
        if let Some(sac) = sacrifice {
            let ok = self.battlefield.iter().any(|c| {
                c.id == sac
                    && c.controller == p
                    && (c.definition.is_artifact()
                        || c.definition.is_enchantment()
                        || c.is_token)
            });
            if !ok {
                return Err(GameError::InvalidTarget);
            }
            if let Some(c) = self.dying_snapshot(sac) {
                self.died_card_snapshots.insert(sac, c);
            }
            events.push(GameEvent::PermanentSacrificed { card_id: sac, who: p });
            let mut die = self.remove_to_graveyard_with_triggers(sac);
            events.append(&mut die);
            if let Some(c) = self.players[p].hand.iter_mut().find(|c| c.id == card_id) {
                c.bargained = true;
            }
            // CR 702.176 — "this spell costs {N} less if it's bargained"
            // (Ice Out, Johann's Stopgap), threaded through the cast path.
            self.extra_cast_reduction = self.players[p]
                .hand
                .iter()
                .find(|c| c.id == card_id)
                .and_then(|c| {
                    c.definition.static_abilities.iter().find_map(|sa| match sa.effect {
                        StaticEffect::BargainCostReduction { amount } => Some(amount),
                        _ => None,
                    })
                })
                .unwrap_or(0);
        }
        let cast = self.cast_spell(card_id, target, additional_targets, mode, x_value);
        self.extra_cast_reduction = 0;
        let mut cast = cast?;
        events.append(&mut cast);
        Ok(events)
    }

    /// CR 702.172 — cast a Spree spell, choosing one or more modes. Each
    /// chosen mode's mana cost is an additional cost (folded into the total
    /// in `cast_spell_with_convoke`), and the chosen indices are stamped onto
    /// the resolving spell so `Effect::Spree` runs exactly those modes.
    pub(crate) fn cast_spell_spree(
        &mut self,
        card_id: CardId,
        spree_modes: Vec<u8>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        // The card must be a Spree / Tiered / ChooseModesCast spell in hand;
        // validate the modes. Selection bounds per variant:
        // - Spree (CR 702.172a): 1..=all, distinct.
        // - Tiered (FIN "Choose one additional cost."): exactly one.
        // - ChooseModesCast: `min..=max`, repeats iff `allow_repeats`
        //   (Choreographed Sparks "one or both"; Moment of Reckoning
        //   "up to four, same mode more than once").
        // - ChooseModesByPoints (the BLB Season cycle): any picks whose
        //   point prices total at most the printed budget, repeats allowed.
        let (mode_count, min_pick, max_pick, allow_repeats, points) = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| match &c.definition.effect {
                crate::effect::Effect::Spree { modes } => {
                    Some((modes.len(), 1usize, modes.len(), false, None))
                }
                crate::effect::Effect::Tiered { modes } => Some((modes.len(), 1, 1, false, None)),
                crate::effect::Effect::ChooseModesCast { modes, min, max, allow_repeats } => {
                    Some((modes.len(), *min as usize, *max as usize, *allow_repeats, None))
                }
                crate::effect::Effect::ChooseModesByPoints { modes, points, budget } => Some((
                    modes.len(),
                    0,
                    modes.len() * *budget as usize,
                    true,
                    Some((points.clone(), *budget)),
                )),
                _ => None,
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        // In range, distinct unless repeats are allowed, kept in printed
        // order so target slots line up with resolution.
        let mut chosen: Vec<u8> = Vec::new();
        for &i in &spree_modes {
            if (i as usize) < mode_count && (allow_repeats || !chosen.contains(&i)) {
                chosen.push(i);
            }
        }
        chosen.sort_unstable();
        if chosen.len() < min_pick || chosen.len() > max_pick {
            return Err(GameError::InvalidTarget);
        }
        if let Some((prices, budget)) = points {
            let spent: u32 =
                chosen.iter().map(|i| prices.get(*i as usize).copied().unwrap_or(0) as u32).sum();
            if spent > budget as u32 {
                return Err(GameError::InvalidTarget);
            }
        }
        self.cast_atomically(|g| {
            g.pending_spree_modes = Some(chosen.clone());
            let cast =
                g.cast_spell(card_id, target.clone(), additional_targets.clone(), None, x_value);
            g.pending_spree_modes = None;
            cast
        })
    }

    /// CR 702.27 — cast a spell paying its optional Buyback cost. The
    /// resolving spell returns to its owner's hand instead of the
    /// graveyard (`continue_spell_resolution` consults `card.bought_back`).
    pub(crate) fn cast_spell_buyback(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[], &[], CastFlags { buyback: true, ..Default::default() })
    }

    /// CR 709.5 — cast a Room card's chosen door for that door's cost. The
    /// permanent enters with that door unlocked.
    pub(crate) fn cast_room_door(
        &mut self,
        card_id: CardId,
        right: bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let is_room = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| c.definition.room.is_some())
            .ok_or(GameError::CardNotInHand(card_id))?;
        if !is_room {
            return Err(GameError::CardNotInHand(card_id));
        }
        self.cast_spell_with_convoke(
            card_id, None, vec![], None, None, &[], &[],
            CastFlags { room_door: Some(u8::from(right)), ..Default::default() },
        )
    }

    /// CR 709.5e / 116.2m — special action: pay a locked door's cost at
    /// sorcery speed (main phase, empty stack) to unlock it.
    pub fn unlock_room_door(
        &mut self,
        card_id: CardId,
        right: bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        let cost = {
            let card = self
                .battlefield
                .iter()
                .find(|c| c.id == card_id && c.controller == p)
                .ok_or(GameError::CardNotOnBattlefield(card_id))?;
            let room = card.definition.room.as_deref().ok_or(GameError::InvalidTarget)?;
            let bit = if right { 2u8 } else { 1u8 };
            if card.unlocked_doors & bit != 0 {
                return Err(GameError::InvalidTarget);
            }
            if right { room.right.cost.clone() } else { room.left.cost.clone() }
        };
        let forced_only = self.players[p].manual_mana;
        let kind =
            crate::mana::SpellKind { room_or_door: true, ..crate::mana::SpellKind::default() };
        let receipt = self.try_pay_with_auto_tap_kind(p, &cost, forced_only, &kind)?;
        let mut events = receipt.auto_events;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        self.set_room_door_unlocked(card_id, right, &mut events);
        Ok(events)
    }

    /// CR 709.5c/h — give a Room permanent the unlocked designation for one
    /// door, rebuild its live definition, and fire that door's "when you
    /// unlock this door" triggers.
    pub(crate) fn set_room_door_unlocked(
        &mut self,
        card_id: CardId,
        right: bool,
        events: &mut Vec<GameEvent>,
    ) {
        let Some(card) = self.battlefield_find_mut(card_id) else { return };
        if !card.unlock_room_door(right) {
            return;
        }
        let controller = card.controller;
        // DSK Eerie (CR 709.5) — both doors now open: "you fully unlock a
        // Room". Surface the event so Eerie triggers fire via normal dispatch.
        if card.unlocked_doors == 0b11 {
            events.push(GameEvent::RoomFullyUnlocked { room: card_id, controller });
        }
        let unlock_triggers: Vec<crate::effect::Effect> = card
            .definition
            .room
            .as_deref()
            .map(|room| {
                let door = if right { &room.right } else { &room.left };
                door.triggered_abilities
                    .iter()
                    .filter(|t| t.event.kind == crate::card::EventKind::DoorUnlocked)
                    .map(|t| t.effect.clone())
                    .collect()
            })
            .unwrap_or_default();
        for effect in unlock_triggers {
            let auto_target = self.auto_target_for_effect(&effect, controller);
            self.stack.push(
                TriggerPush::new(card_id, controller, effect)
                    .target(auto_target)
                    .build(),
            );
        }
    }

    /// CR 709.5c — re-lock one door of a Room permanent, rebuilding its live
    /// definition from the remaining unlocked designations.
    pub fn relock_room_door(&mut self, card_id: CardId, right: bool) {
        let Some(card) = self.battlefield_find_mut(card_id) else { return };
        let bit = if right { 2u8 } else { 1u8 };
        if card.definition.room.is_none() || card.unlocked_doors & bit == 0 {
            return;
        }
        card.unlocked_doors &= !bit;
        let doors = card.unlocked_doors;
        card.definition = std::sync::Arc::new(card.definition.room_definition_with(doors));
    }

    /// CR 702.41 — cast a modal spell paying its Entwine cost; every mode
    /// runs in order at resolution.
    pub(crate) fn cast_spell_entwine(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[], &[], CastFlags { entwine: true, ..Default::default() })
    }

    /// CR 702.62 — suspend a card from hand: pay its suspend cost and exile
    /// it with N time counters. Timing follows the card's normal cast
    /// timing (sorcery-speed unless the card is instant-speed). Removal +
    /// the free cast happen later in `process_suspend`.
    pub(crate) fn suspend_card(&mut self, card_id: CardId) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::{CounterType, Keyword};
        let p = self.priority.player_with_priority;
        // Locate the card in the priority player's hand and its Suspend params.
        let (n, cost, is_instant) = {
            let card = self.players[p]
                .hand
                .iter()
                .find(|c| c.id == card_id)
                .ok_or(GameError::CardNotInHand(card_id))?;
            let suspend = card.definition.keywords.iter().find_map(|k| match k {
                Keyword::Suspend(n, cost) => Some((*n, cost.clone())),
                _ => None,
            });
            let Some((n, cost)) = suspend else {
                return Err(GameError::CardNotInHand(card_id));
            };
            (n, cost, card.definition.is_instant_speed())
        };
        // CR 702.62a — you may suspend only when you could cast the card.
        if !is_instant && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // Pay the suspend cost.
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        let mut events = receipt.auto_events;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        // Exile from hand with N time counters.
        let mut card = self
            .players[p]
            .remove_from_hand(card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        card.add_counters(CounterType::Time, n);
        self.exile.push(card);
        events.push(GameEvent::PermanentExiled { card_id });
        events.push(GameEvent::CounterAdded {
            card_id,
            counter_type: CounterType::Time,
            count: n,
        });
        Ok(events)
    }

    /// CR 702.143 — foretell a card from hand: pay {2} and exile it
    /// face-down. Sorcery-speed only (CR 702.143b). The card can be cast for
    /// its foretell cost on a later turn via `cast_foretold`.
    pub(crate) fn foretell_card(&mut self, card_id: CardId) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let has_foretell = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| c.definition.foretell_cost.is_some())
            .ok_or(GameError::CardNotInHand(card_id))?;
        if !has_foretell {
            return Err(GameError::CardNotInHand(card_id));
        }
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // The foretell *action* always costs {2}.
        let cost = crate::mana::ManaCost {
            symbols: vec![crate::mana::ManaSymbol::Generic(2)],
        };
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        let mut events = receipt.auto_events;
        let mut card = self
            .players[p]
            .remove_from_hand(card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        card.face_down = true;
        self.exile.push(card);
        self.foretold_this_turn.insert(card_id);
        events.push(GameEvent::PermanentExiled { card_id });
        Ok(events)
    }

    /// CR 702.36b — cast a card with Morph face down as a 2/2 creature for {3}.
    /// The card is set face down (its real definition stashed) before going on
    /// the stack, so it's a nameless colorless 2/2 creature spell and enters the
    /// battlefield face down; turn it up later for its Morph cost.
    /// CR 702.36b — the generic mana a face-down cast costs `seat`: the flat
    /// {3}, less every `FaceDownSpellsCostLess` static they control (Dream
    /// Chisel) and any turn-scoped grant (Goblin Maskmaker). Surfaced as
    /// `PlayerView.face_down_cast_cost`.
    pub fn face_down_cast_cost(&self, seat: usize) -> u32 {
        let reduction: u32 = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .flat_map(|c| &c.definition.static_abilities)
            .filter_map(|sa| match sa.effect {
                crate::effect::StaticEffect::FaceDownSpellsCostLess { amount } => Some(amount),
                _ => None,
            })
            .sum();
        // `seat` can be the spectator sentinel, which indexes no player.
        let turn_grant =
            self.players.get(seat).map_or(0, |p| p.face_down_discount_this_turn);
        3u32.saturating_sub(reduction + turn_grant)
    }

    pub(crate) fn cast_face_down(&mut self, card_id: CardId) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let has_morph = self
            .players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| {
                c.definition.keywords.iter().any(|k| {
                    matches!(
                        k,
                        Keyword::Morph(_)
                            | Keyword::MorphCost(_)
                            | Keyword::Megamorph(_)
                            | Keyword::Disguise(_)
                    )
                })
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        if !has_morph {
            return Err(GameError::CardNotInHand(card_id));
        }
        // Morph is a creature cast: sorcery-speed timing.
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        let cost = crate::mana::ManaCost {
            symbols: vec![crate::mana::ManaSymbol::Generic(self.face_down_cast_cost(p))],
        };
        let forced_only = self.players[p].manual_mana;
        let kind = crate::mana::SpellKind {
            creature: true,
            casting_nonartifact_spell: true,
            colorless: true,
            face_down: true,
            ..Default::default()
        };
        let receipt = self.try_pay_with_auto_tap_kind(p, &cost, forced_only, &kind)?;
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        let mut card = self
            .players[p]
            .remove_from_hand(card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        card.turn_face_down();
        let mut events = receipt.auto_events;
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(p, card, None, vec![], None, 0, 0, mana_spent, true);
        Ok(events)
    }

    /// CR 708.5 — turn a face-down permanent face up. Special action (no
    /// stack): pay its Morph/Megamorph cost, or — for a manifested creature
    /// card — its mana cost. Restores the real definition and fires
    /// `EventKind::TurnedFaceUp`.
    pub(crate) fn turn_face_up_action(
        &mut self,
        card_id: CardId,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.turn_face_up_for_x(card_id, 0)
    }

    /// CR 702.36b / 708.7 — the mana cost `seat` must pay to turn their
    /// face-down permanent `card_id` face up, with `x_value` substituted into
    /// any `{X}`: the Morph / Megamorph / Disguise cost (plus every Exiled
    /// Doomsayer surcharge, less any `disguise_cost_reduction_per`), or a
    /// manifested creature card's own mana cost. `None` when the permanent
    /// isn't a face-down permanent `seat` controls, or can't be turned up at
    /// all (a manifested noncreature). Shared by the action path and the
    /// `PermanentView.turn_up_cost_label` projection.
    pub fn turn_up_mana_cost(
        &self,
        seat: usize,
        card_id: CardId,
        x_value: u32,
    ) -> Option<crate::mana::ManaCost> {
        let c = self
            .battlefield
            .iter()
            .find(|c| c.id == card_id && c.controller == seat && c.face_down)?;
        let real = c.face_up_def.as_ref()?;
        // Morph / Megamorph / Disguise takes precedence; otherwise a
        // manifested creature card turns up for its mana cost.
        let morph_cost = real.keywords.iter().find_map(|kw| match kw {
            Keyword::Morph(mc) | Keyword::Megamorph(mc) | Keyword::Disguise(mc) => Some(mc.clone()),
            _ => None,
        });
        // CR 702.36b — Exiled Doomsayer taxes every turn-up cost.
        let morph_tax: u32 = self
            .battlefield
            .iter()
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match sa.effect {
                crate::effect::StaticEffect::MorphCostsMore { amount } => Some(amount),
                _ => None,
            })
            .sum();
        // "This cost is reduced by {1} for each …" (Fugitive Codebreaker).
        let morph_discount = real
            .disguise_cost_reduction_per
            .as_ref()
            .map(|v| {
                let ctx = crate::game::effects::EffectContext::for_ability(card_id, seat, None);
                self.evaluate_value(v, &ctx).max(0) as u32
            })
            .unwrap_or(0);
        match morph_cost {
            Some(mc) => {
                let mut c = mc.with_x_value(x_value).plus_generic(morph_tax);
                c.reduce_generic(morph_discount);
                Some(c)
            }
            None if real.is_creature() => Some(real.cost.clone()),
            None => None,
        }
    }

    /// [`turn_face_up_action`] paying `x_value` into an `{X}` in the morph cost
    /// (CR 702.36b — Warbreak Trumpeter). The paid X is stamped on the
    /// permanent so the turn-up trigger can read it via `Value::XFromCost`.
    ///
    /// [`turn_face_up_action`]: Self::turn_face_up_action
    pub(crate) fn turn_face_up_for_x(
        &mut self,
        card_id: CardId,
        x_value: u32,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        // CR 702.36b — "Morph—[non-mana cost]": pay it through the shared
        // ward-cost payer instead of the mana path. The Exiled Doomsayer tax
        // is a mana surcharge and doesn't apply to these.
        let alt_morph = self
            .battlefield
            .iter()
            .find(|c| c.id == card_id && c.controller == p && c.face_down)
            .and_then(|c| c.face_up_def.as_ref())
            .and_then(|d| {
                d.keywords.iter().find_map(|kw| match kw {
                    Keyword::MorphCost(wc) => Some((**wc).clone()),
                    _ => None,
                })
            });
        if let Some(wc) = alt_morph {
            let ctx = crate::game::effects::EffectContext::for_ability(card_id, p, None);
            let mut events = vec![];
            if !self.try_pay_ward_cost(p, &wc, &ctx, &mut events) {
                return Err(GameError::InvalidTarget);
            }
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id) {
                c.turn_face_up();
                c.cast_x_value = x_value;
            }
            events.push(GameEvent::TurnedFaceUp { card_id });
            return Ok(events);
        }
        let cost = self
            .turn_up_mana_cost(p, card_id, x_value)
            .ok_or(GameError::InvalidTarget)?;
        // CR 702.36e — Megamorph turns the permanent up with a +1/+1 counter.
        let megamorph = self
            .battlefield
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| c.face_up_def.as_ref())
            .map(|d| d.keywords.iter().any(|k| matches!(k, Keyword::Megamorph(_))))
            .unwrap_or(false);
        let forced_only = self.players[p].manual_mana;
        let kind = crate::mana::SpellKind { turning_face_up: true, ..Default::default() };
        let receipt = self.try_pay_with_auto_tap_kind(p, &cost, forced_only, &kind)?;
        let mut events = receipt.auto_events;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id) {
            c.turn_face_up();
            c.cast_x_value = x_value;
            if megamorph {
                c.add_counters(crate::card::CounterType::PlusOnePlusOne, 1);
            }
        }
        // The returned events are dispatched once by `perform_action`; an extra
        // internal dispatch here double-fired turn-up triggers (CR 603.2).
        events.push(GameEvent::TurnedFaceUp { card_id });
        Ok(events)
    }

    /// CR 702.143c — cast a foretold card from exile for its foretell cost.
    /// Legal only on a turn after the card was foretold; timing follows the
    /// card's normal cast timing.
    pub(crate) fn cast_foretold(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let pos = self
            .exile
            .iter()
            .position(|c| c.id == card_id && c.face_down && c.owner == p)
            .ok_or(GameError::CardNotInHand(card_id))?;
        // CR 702.143b — not on the turn it was foretold.
        if self.foretold_this_turn.contains(&card_id) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // CR 601 — Drannith Magistrate forbids casting from any non-hand zone.
        if self.cast_from_zone_blocked(p, &self.exile[pos].definition, crate::card::Zone::Exile) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let foretell_cost = self.exile[pos]
            .definition
            .foretell_cost
            .clone()
            .ok_or(GameError::SorcerySpeedOnly)?;
        let is_instant = self.exile[pos].definition.is_instant_speed();
        let must_be_sorcery_speed = !is_instant || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
        }
        let mut cost = if foretell_cost.has_x() {
            foretell_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            foretell_cost
        };
        let reduction =
            cost_reduction_for_spell_full(self, p, &self.exile[pos], target.as_ref(), false, true);
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        apply_spell_cost_floor(self, &mut cost);
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        // Re-locate by id: payment ran after `pos` was captured.
        let mut card = Self::take_card(&mut self.exile, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        card.face_down = false;
        let mut events = receipt.auto_events;
        events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: CastFace::Front,
        });
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            mana_spent,
            false,
        );
        Ok(events)
    }

    /// CR 715 — cast the instant/sorcery adventure half of a card from hand.
    /// Pay the adventure cost; on resolution the card is exiled (with
    /// `on_adventure` set) instead of going to the graveyard, so the creature
    /// half can be cast from exile later via `cast_adventure_creature`.
    pub fn cast_adventure(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let adv = self
            .players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| c.definition.has_adventure().cloned())
            .ok_or(GameError::CardNotInHand(card_id))?;
        // An adventure spell is a noncreature spell — respect a Ranger-Captain
        // of Eos style lock.
        if self.players[p].cant_cast_noncreature_this_turn {
            return Err(GameError::CantCastNoncreature);
        }
        let must_be_sorcery_speed = !adv.is_instant_speed() || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if let Some(ref tgt) = target {
            self.check_target_legality_with_source(tgt, p, Some(card_id))?;
        }
        let mut cost = if adv.cost.has_x() {
            adv.cost.with_x_value(x_value.unwrap_or(0))
        } else {
            adv.cost.clone()
        };
        apply_spell_cost_floor(self, &mut cost);
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        let mut card = self.players[p].remove_from_hand(card_id).unwrap();
        card.cast_from_hand = true;
        card.cast_from_exile = false;
        card.cast_from_library = self.casting_from_library_top == Some(card_id);
        card.adventuring = true;
        let mut events = receipt.auto_events;
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            mana_spent,
            true,
        );
        Ok(events)
    }

    /// CR 702.183 — cast a card's Omen half from hand for its Omen cost. The
    /// card becomes an instant/sorcery spell; on resolution or counter it is
    /// shuffled into its owner's library instead of going to the graveyard.
    pub(crate) fn cast_omen(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let omen = self
            .players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| c.definition.has_omen().cloned())
            .ok_or(GameError::CardNotInHand(card_id))?;
        // An Omen spell is a noncreature spell — respect a noncreature lock.
        if self.players[p].cant_cast_noncreature_this_turn {
            return Err(GameError::CantCastNoncreature);
        }
        let must_be_sorcery_speed = !omen.is_instant_speed() || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if let Some(ref tgt) = target {
            self.check_target_legality_with_source(tgt, p, Some(card_id))?;
        }
        let mut cost = if omen.cost.has_x() {
            omen.cost.with_x_value(x_value.unwrap_or(0))
        } else {
            omen.cost.clone()
        };
        apply_spell_cost_floor(self, &mut cost);
        let forced_only = self.players[p].manual_mana;
        // The Omen half is a noncreature instant/sorcery cast; flag it so
        // Omen-restricted mana (Maelstrom of the Spirit Dragon) may fund it.
        let kind = crate::mana::SpellKind {
            instant_or_sorcery: true,
            casting_nonartifact_spell: true,
            omen: true,
            ..Default::default()
        };
        let receipt = self.try_pay_with_auto_tap_kind(p, &cost, forced_only, &kind)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        let mut card = self.players[p].remove_from_hand(card_id).unwrap();
        card.cast_from_hand = true;
        card.cast_from_exile = false;
        card.cast_from_library = self.casting_from_library_top == Some(card_id);
        card.omen_casting = true;
        let mut events = receipt.auto_events;
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            mana_spent,
            true,
        );
        Ok(events)
    }

    /// CR 715 — cast the creature half of a card that's in exile after going
    /// on an adventure. Pays the card's regular mana cost.
    pub(crate) fn cast_adventure_creature(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let pos = self
            .exile
            .iter()
            .position(|c| c.id == card_id && c.on_adventure && c.owner == p)
            .ok_or(GameError::CardNotInHand(card_id))?;
        // CR 601 — Drannith Magistrate forbids casting from any non-hand zone.
        if self.cast_from_zone_blocked(p, &self.exile[pos].definition, crate::card::Zone::Exile) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let is_instant = self.exile[pos].definition.is_instant_speed();
        let must_be_sorcery_speed = !is_instant || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if let Some(ref tgt) = target {
            self.check_target_legality_with_source(tgt, p, Some(card_id))?;
        }
        let base = self.exile[pos].definition.cost.clone();
        let mut cost = if base.has_x() {
            base.with_x_value(x_value.unwrap_or(0))
        } else {
            base
        };
        let reduction =
            cost_reduction_for_spell_full(self, p, &self.exile[pos], target.as_ref(), false, true);
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        apply_spell_cost_floor(self, &mut cost);
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        // Re-locate by id: payment ran after `pos` was captured.
        let mut card = Self::take_card(&mut self.exile, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        card.on_adventure = false;
        card.adventuring = false;
        card.cast_from_hand = false;
        let mut events = receipt.auto_events;
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            mana_spent,
            true,
        );
        Ok(events)
    }

    /// CR 709 — cast the right half (or, with `fused`, both halves) of a split
    /// card from hand. The left half is cast through the normal cast path
    /// (the main definition fields describe it); this handles the right and
    /// fused casts. On resolution the card goes to the graveyard like any
    /// spell (handled by the spell resolver via the `split_cast` marker).
    pub(crate) fn cast_split_half(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        fused: bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let split = self
            .players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| c.definition.has_split().cloned())
            .ok_or(GameError::CardNotInHand(card_id))?;
        if fused && !split.fuse {
            return Err(GameError::NotFusable);
        }
        // CR 702.127a — an Aftermath half can be cast only from the graveyard
        // (`CastAftermath`), never from hand.
        if split.aftermath {
            return Err(GameError::InvalidTarget);
        }
        // Both halves of a split card are instant/sorcery (noncreature) — a
        // Ranger-Captain of Eos lock forbids casting them.
        if self.players[p].cant_cast_noncreature_this_turn {
            return Err(GameError::CantCastNoncreature);
        }
        // Sorcery-speed gate: the (fused) cast is instant-speed only when
        // every half being cast can be cast at instant speed.
        let left_instant = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| c.definition.card_types.contains(&CardType::Instant))
            .unwrap_or(false);
        let all_instant = if fused {
            left_instant && split.right.is_instant_speed()
        } else {
            split.right.is_instant_speed()
        };
        let must_be_sorcery_speed = !all_instant || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if let Some(ref tgt) = target {
            self.check_target_legality_with_source(tgt, p, Some(card_id))?;
        }
        for tgt in &additional_targets {
            self.check_target_legality_with_source(tgt, p, Some(card_id))?;
        }
        // Build the cost: the right half's cost, plus the left half's cost
        // when fused (CR 702.102 — pay both costs).
        let mut cost = split.right.cost.clone();
        if fused {
            let left_cost = self.players[p]
                .hand
                .iter()
                .find(|c| c.id == card_id)
                .map(|c| c.definition.cost.clone())
                .unwrap_or_default();
            cost.symbols.extend(left_cost.symbols);
        }
        if cost.has_x() {
            cost = cost.with_x_value(x_value.unwrap_or(0));
        }
        apply_spell_cost_floor(self, &mut cost);
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        let mut card = self.players[p].remove_from_hand(card_id).unwrap();
        card.cast_from_hand = true;
        card.cast_from_exile = false;
        card.cast_from_library = self.casting_from_library_top == Some(card_id);
        card.split_cast = Some(if fused { 2 } else { 1 });
        let mut events = receipt.auto_events;
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            mana_spent,
            true,
        );
        Ok(events)
    }

    /// CR 702.127 — cast the Aftermath (right) half of a split card from the
    /// owner's graveyard. Pays the right half's cost; on resolution the card
    /// is exiled (handled by the spell resolver via `split_cast` + the
    /// `aftermath` flag).
    pub(crate) fn cast_aftermath(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let pos = self
            .players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInGraveyard(card_id))?;
        let split = self.players[p].graveyard[pos]
            .definition
            .has_split()
            .filter(|s| s.aftermath)
            .cloned()
            .ok_or(GameError::InvalidTarget)?;
        if self.players[p].cant_cast_noncreature_this_turn {
            return Err(GameError::CantCastNoncreature);
        }
        let must_be_sorcery_speed =
            !split.right.is_instant_speed() || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if let Some(ref tgt) = target {
            self.check_target_legality_with_source(tgt, p, Some(card_id))?;
        }
        let mut cost = if split.right.cost.has_x() {
            split.right.cost.with_x_value(x_value.unwrap_or(0))
        } else {
            split.right.cost.clone()
        };
        apply_spell_cost_floor(self, &mut cost);
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        // Re-locate by id: payment ran after `pos` was captured.
        let mut card = Self::take_card(&mut self.players[p].graveyard, card_id)
            .ok_or(GameError::CardNotInGraveyard(card_id))?;
        card.cast_from_hand = false;
        card.split_cast = Some(1);
        let mut events = receipt.auto_events;
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            mana_spent,
            false,
        );
        Ok(events)
    }

    /// CR 702.170 — plot a card from hand: pay its plot cost and exile it
    /// face-up. Special action; main phase + empty stack only (sorcery speed).
    pub(crate) fn plot_card(&mut self, card_id: CardId) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        // CR 702.170 — Fblthp, Lost on the Range grants the top card of your
        // library plot for its mana cost; that card is plotted from the
        // library rather than from hand.
        let from_library_top = self.may_plot_from_library_top(p)
            && self.players[p].library.first().is_some_and(|c| {
                c.id == card_id && !c.definition.card_types.contains(&CardType::Land)
            });
        let mut cost = if from_library_top {
            self.players[p].library[0].definition.cost.clone()
        } else {
            self.players[p]
                .hand
                .iter()
                .find(|c| c.id == card_id)
                .and_then(|c| c.definition.plot_cost.clone())
                .ok_or(GameError::CardNotInHand(card_id))?
        };
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // Doc Aurlock — "Plotting cards from your hand costs {2} less."
        let plot_reduction: u32 = self
            .battlefield
            .iter()
            .filter(|s| s.controller == p)
            .flat_map(|s| &s.definition.static_abilities)
            .filter_map(|sa| match sa.effect {
                crate::effect::StaticEffect::PlotCostReduction { amount } => Some(amount),
                _ => None,
            })
            .sum();
        if plot_reduction > 0 {
            cost.reduce_generic(plot_reduction);
        }
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let mut events = receipt.auto_events;
        let card = if from_library_top {
            self.players[p].library.remove(0)
        } else {
            self.players[p]
                .remove_from_hand(card_id)
                .ok_or(GameError::CardNotInHand(card_id))?
        };
        // CR 702.170 — "When this card becomes plotted, …" self-triggers fire
        // from exile as the card is plotted (Aloe Alchemist, Longhorn
        // Sharpshooter). Gather them off the card before it settles in exile.
        let plot_triggers: Vec<crate::effect::Effect> = card
            .definition
            .triggered_abilities
            .iter()
            .filter(|t| t.event.kind == crate::effect::EventKind::BecomesPlotted)
            .map(|t| t.effect.clone())
            .collect();
        self.exile.push(card);
        self.plotted_cards.insert(card_id);
        self.plotted_this_turn.insert(card_id);
        events.push(GameEvent::PermanentExiled { card_id });
        self.push_plot_triggers(card_id, p, plot_triggers);
        Ok(events)
    }

    /// CR 702.170 — fire a freshly-plotted card's "when this becomes plotted"
    /// self-triggers from exile. `ZoneDest::ExilePlotted` gathers them itself.
    pub(crate) fn fire_becomes_plotted_triggers(&mut self, card_id: CardId, controller: usize) {
        let effects: Vec<crate::effect::Effect> = self
            .exile
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| {
                c.definition
                    .triggered_abilities
                    .iter()
                    .filter(|t| t.event.kind == crate::effect::EventKind::BecomesPlotted)
                    .map(|t| t.effect.clone())
                    .collect()
            })
            .unwrap_or_default();
        self.push_plot_triggers(card_id, controller, effects);
    }

    fn push_plot_triggers(
        &mut self,
        card_id: CardId,
        controller: usize,
        effects: Vec<crate::effect::Effect>,
    ) {
        for effect in effects {
            let auto_target =
                self.auto_target_for_effect_avoiding(&effect, controller, Some(card_id));
            self.push_pending_trigger(
                crate::game::types::PendingTriggerPush {
                    from_mana_ability: false,
                    actor: None,
                    source: card_id,
                    controller,
                    effect,
                    subject: Some(crate::game::effects::EntityRef::Card(card_id)),
                    event_amount: 0,
                    mode: None,
                    intervening_if: None,
                },
                auto_target,
            );
        }
    }

    /// True when `p` controls a `MayPlotFromLibraryTop` grant (Fblthp).
    pub(crate) fn may_plot_from_library_top(&self, p: usize) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::MayPlotFromLibraryTop)
                })
        })
    }

    /// CR 702.170d — cast a plotted card from exile without paying its mana
    /// cost. Sorcery speed; legal only on a turn after it was plotted.
    pub(crate) fn cast_plotted(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let pos = self
            .exile
            .iter()
            .position(|c| c.id == card_id && c.owner == p)
            .filter(|_| self.plotted_cards.contains(&card_id))
            .ok_or(GameError::CardNotInHand(card_id))?;
        // CR 702.170d — only as a sorcery, and not the turn it was plotted.
        if self.plotted_this_turn.contains(&card_id) || !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // CR 601 — Drannith Magistrate forbids casting from any non-hand zone.
        if self.cast_from_zone_blocked(p, &self.exile[pos].definition, crate::card::Zone::Exile) {
            return Err(GameError::CardNotInHand(card_id));
        }
        if let Some(ref tgt) = target {
            self.check_target_legality_with_source(tgt, p, Some(card_id))?;
        }
        // Re-locate by id at removal time (target checks ran in between).
        let card = Self::take_card(&mut self.exile, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        self.plotted_cards.remove(&card_id);
        let events = vec![GameEvent::SpellCast { player: p, card_id, face: CastFace::Front }];
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            0,
            false,
        );
        Ok(events)
    }

    /// CR 702.103 — cast an enchantment-creature for its Bestow cost as an
    /// Aura targeting a creature (`target`). The resolving permanent enters
    /// attached and is an Aura (not a creature) while bestowed.
    pub(crate) fn cast_bestow(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[], &[], CastFlags { bestow: true, ..Default::default() })
    }

    /// Cast a spell with `Keyword::Delve` (CR 702.66), exiling each card in
    /// `delve_cards` from the caster's graveyard to pay {1} of the spell's
    /// generic cost. Each listed card must be in the caster's graveyard and
    /// the spell must have `Keyword::Delve`. The graveyard cards are not
    /// physically exiled until the (reduced) mana cost is successfully paid,
    /// so a failed payment leaves the graveyard untouched.
    pub(crate) fn cast_spell_with_delve(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        delve_cards: &[CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &[], delve_cards, CastFlags::default())
    }

    /// CR 701.67 — cast a spell paying its "waterbend {N}" additional cost.
    /// `helpers` are untapped artifacts/creatures the caster controls (count
    /// clamped to N); each taps for {1} of the waterbend generic, the rest
    /// comes from real mana. For an optional "you may waterbend {N}" rider this
    /// is the pay branch and stamps `cast_via_waterbend`.
    pub(crate) fn cast_spell_waterbend(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        helpers: &[CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let Some(card) = self.players[p].hand.iter().find(|c| c.id == card_id) else {
            return Err(GameError::CardNotInHand(card_id));
        };
        let Some((amt, _optional)) = waterbend_amount(&card.definition, x_value) else {
            return Err(GameError::SorcerySpeedOnly); // reuse: card has no waterbend
        };
        self.cast_spell_with_convoke(
            card_id,
            target,
            additional_targets,
            mode,
            x_value,
            helpers,
            &[],
            CastFlags { waterbend: Some(amt), ..Default::default() },
        )
    }

    /// CR 702.51 — true when a `StaticEffect::GrantConvokeToSpells` permanent
    /// `p` controls (Chief Engineer) grants convoke to this spell.
    pub(crate) fn spell_granted_convoke(&self, p: usize, card: &CardInstance) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition.static_abilities.iter().any(|sa| match &sa.effect {
                    crate::effect::StaticEffect::GrantConvokeToSpells { filter } => {
                        crate::game::layers::requirement_matches_card(filter, card, p)
                    }
                    _ => false,
                })
        })
    }

    /// Internal cast-spell helper with optional convoke creatures and delve
    /// cards. Each convoke creature must be untapped + controlled by the
    /// caster + the spell must have `Keyword::Convoke`; each tap adds {1}
    /// generic mana to the player's pool. Each delve card must be in the
    /// caster's graveyard + the spell must have `Keyword::Delve`; each one
    /// exiled reduces the generic cost by {1} (CR 702.66).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cast_spell_with_convoke(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        convoke_creatures: &[CardId],
        delve_cards: &[CardId],
        flags: CastFlags,
    ) -> Result<Vec<GameEvent>, GameError> {
        let CastFlags { kicked, buyback, bestow, entwine, room_door, gift, mut waterbend } = flags;
        let p = self.priority.player_with_priority;

        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }

        // Meddling Mage — spells with the chosen name can't be cast.
        let spell_name = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .map(|c| c.definition.name);
        if let Some(name) = spell_name
            && self.battlefield.iter().any(|c| {
                c.named_card.as_deref() == Some(name)
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(sa.effect, crate::effect::StaticEffect::NamedSpellCantBeCast)
                    })
            })
        {
            return Err(GameError::SpellNameLocked);
        }
        // Ashiok's Erasure — an opponent of the caster controls a permanent
        // whose `OpponentsCantCastNamed` static locks this spell's name (the
        // card exiled by the Erasure). The lock lives as long as that
        // permanent stays on the battlefield.
        if let Some(name) = spell_name
            && self.battlefield.iter().any(|c| {
                c.named_card.as_deref() == Some(name)
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(sa.effect, crate::effect::StaticEffect::OpponentsCantCastNamed)
                    })
                    && !self.same_team(c.controller, p)
            })
        {
            return Err(GameError::SpellNameLocked);
        }
        // Circu, Dimir Lobotomist — an opponent of the caster controls a
        // permanent, and a card exiled with it shares this spell's name.
        if let Some(name) = spell_name
            && self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::OpponentsCantCastNamesExiledWithSource
                    )
                })
                    && !self.same_team(c.controller, p)
                    && self
                        .exile
                        .iter()
                        .any(|e| e.exiled_with == Some(c.id) && e.definition.name == name)
            })
        {
            return Err(GameError::SpellNameLocked);
        }
        // Academic Probation mode 0 — an opponent of the caster named this
        // spell ("Opponents can't cast spells with the chosen name until your
        // next turn"); the lock lives on the naming player until their turn.
        if let Some(name) = spell_name
            && self.players.iter().enumerate().any(|(i, pl)| {
                !self.same_team(i, p) && pl.opponents_cant_cast_named.iter().any(|n| n == name)
            })
        {
            return Err(GameError::SpellNameLocked);
        }

        // CR 601.2b — interactive additional-cost choices for a hand-paying
        // caster on the plain cast path: which permanent to sacrifice
        // ("sacrifice a …" — Crop Rotation, Reckless Abandon) or which card to
        // discard ("discard a card" — Big Score, Illuminate History) instead of
        // the engine auto-picking. Suspend *here* — before the card leaves hand
        // or any cost is paid — and re-run the whole cast on answer with the
        // pick stashed. Each cost type guards on its own stash so a card with
        // several such costs chains (suspend → replay → suspend). Only a genuine
        // choice is surfaced (more legal options than required); lone options
        // and multi-count costs keep the auto-pick. The alt-cost / convoke /
        // delve paths are excluded so their own choice plumbing isn't disturbed.
        //
        // [`manual_mana`], not `wants_ui`: an additional cost is a cost, and
        // prompting a bot for one livelocks the game (see the {X} prompt in
        // `cast_spell` for the full chain). This was the residual 1-in-96
        // deadlock left after the {X} and ability-cost gates were fixed —
        // Bonfire of the Damned and friends, whose discard cost suspended a
        // cast the bot could not actually pay for.
        if self.players[p].manual_mana
            && convoke_creatures.is_empty()
            && delve_cards.is_empty()
            && !buyback
            && !bestow
            && !entwine
            && room_door.is_none()
        {
            let card_info = self.players[p].hand.iter().find(|c| c.id == card_id).map(|c| {
                let mut costs = c.definition.additional_cast_cost.clone();
                // A kicked cast pays the action kicker too — offer its
                // sacrifice choice alongside the printed additional costs.
                if kicked && let Some(kc) = &c.definition.kicker_action_cost {
                    costs.push(kc.clone());
                }
                (c.definition.name.to_string(), costs)
            });
            if let Some((name, costs)) = card_info {
                for cost in &costs {
                    match cost {
                        crate::card::AdditionalCastCost::SacrificePermanent { filter, count }
                            if *count == 1 && self.pending_cast_sacrifices.is_none() =>
                        {
                            let legal: Vec<Target> = self
                                .battlefield
                                .iter()
                                .filter(|c| {
                                    c.controller == p
                                        && self.evaluate_requirement_static(
                                            filter,
                                            &Target::Permanent(c.id),
                                            p,
                                            None,
                                        )
                                })
                                .map(|c| Target::Permanent(c.id))
                                .collect();
                            if legal.len() > 1 {
                                self.pending_decision = Some(crate::game::types::PendingDecision {
                                    decision: crate::decision::Decision::ChooseTarget {
                                        optional: false,
                                        source: card_id,
                                        legal,
                                        source_name: name.clone(),
                                        description: "choose a permanent to sacrifice".into(),
                                    },
                                    resume: crate::game::types::ResumeContext::CastAdditionalCost {
                                        caster: p,
                                        card_id,
                                        target,
                                        additional_targets,
                                        mode,
                                        x_value,
                                        kicked,
                                    },
                                });
                                return Ok(vec![]);
                            }
                        }
                        crate::card::AdditionalCastCost::ExilePermanent { filter, count }
                            if *count == 1 && self.pending_cast_sacrifices.is_none() =>
                        {
                            let legal: Vec<Target> = self
                                .battlefield
                                .iter()
                                .filter(|c| {
                                    c.controller == p
                                        && self.evaluate_requirement_static(
                                            filter,
                                            &Target::Permanent(c.id),
                                            p,
                                            None,
                                        )
                                })
                                .map(|c| Target::Permanent(c.id))
                                .collect();
                            if legal.len() > 1 {
                                self.pending_decision = Some(crate::game::types::PendingDecision {
                                    decision: crate::decision::Decision::ChooseTarget {
                                        optional: false,
                                        source: card_id,
                                        legal,
                                        source_name: name.clone(),
                                        description: "choose a permanent to exile".into(),
                                    },
                                    resume: crate::game::types::ResumeContext::CastAdditionalCost {
                                        caster: p,
                                        card_id,
                                        target,
                                        additional_targets,
                                        mode,
                                        x_value,
                                        kicked,
                                    },
                                });
                                return Ok(vec![]);
                            }
                        }
                        crate::card::AdditionalCastCost::Discard { count, filter }
                            if *count >= 1 && self.pending_cast_discards.is_none() =>
                        {
                            // The card being cast is still in hand here but is
                            // moving to the stack — exclude it as a discard
                            // option (you can't discard the spell to pay its
                            // own cost). A `filter` (Magmatic Insight's "a land
                            // card") restricts the eligible pitches.
                            let hand: Vec<(CardId, String)> = self.players[p]
                                .hand
                                .iter()
                                .filter(|c| c.id != card_id)
                                .filter(|c| {
                                    filter.as_ref().is_none_or(|f| {
                                        self.evaluate_requirement_on_card(f, c, p)
                                    })
                                })
                                .map(|c| (c.id, c.definition.name.to_string()))
                                .collect();
                            if hand.len() > *count as usize {
                                self.pending_decision = Some(crate::game::types::PendingDecision {
                                    decision: crate::decision::Decision::Discard {
                                        player: p,
                                        count: *count,
                                        hand,
                                    },
                                    resume: crate::game::types::ResumeContext::CastAdditionalCost {
                                        caster: p,
                                        card_id,
                                        target,
                                        additional_targets,
                                        mode,
                                        x_value,
                                        kicked,
                                    },
                                });
                                return Ok(vec![]);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Consume any additional-cost picks from a `CastAdditionalCost` resume
        // now, so they're applied to *this* cast only — a later failure
        // (timing, mana) can't leak them onto the next cast.
        let chosen_sacrifices = self.pending_cast_sacrifices.take();
        let chosen_discards = self.pending_cast_discards.take();
        // CR 601.2g float-spend confirmation answer (None until the player
        // has answered the prompt). Taken up front so a later failure can't
        // leak it onto the next cast.
        let spend_float_choice = self.pending_cast_spend_float.take();

        let mut card = self.players[p].remove_from_hand(card_id).unwrap();
        card.cast_from_hand = true;
        card.cast_from_exile = false;
        card.cast_from_library = self.casting_from_library_top == Some(card_id);
        // CR 701.67 — a mandatory "waterbend {N}" rider is paid even on the
        // plain cast path; only the optional "you may waterbend" form is
        // skippable. When no explicit amount arrived via CastSpellWaterbend,
        // derive it for mandatory cards (helpers stay empty — paid from mana).
        if waterbend.is_none()
            && let Some((amt, false)) = waterbend_amount(&card.definition, x_value)
        {
            waterbend = Some(amt);
        }
        // CR 702.32 — opt-in Kicker. Only stamp `kicked` if the card
        // actually has a kicker cost (mana or action); a mana kicker is
        // folded into the spell's mana cost below, an action kicker joins
        // the additional-cast-cost payment.
        // CR 702.32b — an "and/or" kicker cast carries its chosen option
        // indices instead of a single kicker cost.
        let kicker_options: Vec<u8> = std::mem::take(&mut self.cast_kicker_options)
            .into_iter()
            .filter(|i| (*i as usize) < card.definition.kicker_options.len())
            .collect();
        let kicked = kicked
            && (card.definition.has_kicker().is_some()
                || card.definition.kicker_action_cost.is_some()
                || !kicker_options.is_empty());
        card.kicked = kicked;
        card.kicked_options = kicker_options;
        // CR 702.27 — opt-in Buyback; folded into the cost below and read
        // at resolution to return the spell to hand instead of the gy.
        let buyback = buyback && card.definition.has_buyback().is_some();
        card.bought_back = buyback;
        // CR 702.172 — Spree: stamp the chosen modes so `Effect::Spree` runs
        // exactly those at resolution. Their mana costs fold into the total
        // cost below.
        let spree_modes = self.pending_spree_modes.take().unwrap_or_default();
        card.spree_modes = spree_modes.clone();
        // CR 702.41 — opt-in Entwine; only sticks when the card has it (either
        // a mana cost or a non-mana one — "Entwine—Sacrifice two lands").
        let entwine = entwine
            && (card.definition.has_entwine().is_some()
                || card.definition.entwine_additional_cost.is_some());
        card.entwined = entwine;
        // CR 702.165 — opt-in Gift; only sticks when the card has it. The
        // promised gift carries no mana cost, so nothing folds into the cost.
        card.gift_promised = gift && card.definition.gift.is_some();
        // CR 709.5 — a Room cast remembers which door was cast (reusing the
        // split-cast slot); resolution unlocks that door.
        let room_door = room_door.filter(|_| card.definition.room.is_some());
        if let Some(d) = room_door {
            card.split_cast = Some(d);
        }
        // CR 702.103 — Bestow: cast as an Aura targeting a creature. The
        // bestow cost replaces the regular cost (below); `bestowed` flags
        // the resolving permanent as an Aura that attaches to its target.
        let bestow = bestow && card.definition.has_bestow().is_some();
        if bestow {
            // Bestow requires a creature target up front (the spell's own
            // effect doesn't carry one), so an unbestowable cast reverts.
            let creature_target = matches!(
                target,
                Some(Target::Permanent(tid))
                    if self.battlefield.iter().any(|c| c.id == tid && c.definition.is_creature())
            );
            if !creature_target {
                self.players[p].hand.push(card);
                return Err(GameError::SelectionRequirementViolated);
            }
            card.bestowed = true;
        }

        // Ranger-Captain of Eos lock — this player can't cast noncreature
        // spells for the rest of the turn.
        if self.players[p].cant_cast_noncreature_this_turn
            && !card.definition.is_creature()
        {
            self.players[p].hand.push(card);
            return Err(GameError::CantCastNoncreature);
        }

        // Cease-Fire — a turn-scoped, filtered cast lock.
        if !self.players[p].cant_cast_matching_this_turn.is_empty() {
            let locks = self.players[p].cant_cast_matching_this_turn.clone();
            if locks.iter().any(|f| self.evaluate_requirement_on_card(f, &card, p)) {
                self.players[p].hand.push(card);
                return Err(GameError::CantCastNoncreature);
            }
        }

        // City in a Bottle — a symmetric play-lock binds every seat.
        if self.play_locked_for_all(p, &card) {
            self.players[p].hand.push(card);
            return Err(GameError::SpellNameLocked);
        }

        // Llawan — an opponent's static locks a whole class of spells.
        if self.opponent_locks_cast_of(p, &card) {
            self.players[p].hand.push(card);
            return Err(GameError::CantCastNoncreature);
        }

        // Codie lock — this player can't cast permanent spells.
        if card.definition.is_permanent() && self.player_cant_cast_permanent_spells(p) {
            self.players[p].hand.push(card);
            return Err(GameError::CantCastPermanentSpells);
        }

        // Gaddock Teeg lock — high-mana-value / {X} noncreature spells can't be cast.
        if self.noncreature_spell_cast_locked(&card.definition) {
            self.players[p].hand.push(card);
            return Err(GameError::CantCastNoncreature);
        }

        // Nikya of the Old Ways — its controller can't cast noncreature spells.
        if !card.definition.is_creature() && self.player_cant_cast_noncreature_spells(p) {
            self.players[p].hand.push(card);
            return Err(GameError::CantCastNoncreature);
        }

        // Hymn of the Wilds — its controller can't cast instants or sorceries.
        if (card.definition.is_instant() || card.definition.is_sorcery())
            && self.player_cant_cast_instants_or_sorceries(p)
        {
            self.players[p].hand.push(card);
            return Err(GameError::CantCastNoncreature);
        }

        // Hand to Hand — nobody casts instants during combat.
        if card.definition.is_instant() && self.combat_spell_lock_active() {
            self.players[p].hand.push(card);
            return Err(GameError::CantCastNoncreature);
        }

        // Grid Monitor — its controller can't cast creature spells.
        if card.definition.is_creature() && self.player_cant_cast_creature_spells(p) {
            self.players[p].hand.push(card);
            return Err(GameError::CantCastNoncreature);
        }

        // Validate convoke/improvise helpers up-front (before any state
        // mutation). Convoke taps untapped creatures (CR 702.52); Improvise
        // taps untapped artifacts (CR 702.126); each pays {1}.
        let has_convoke = card.definition.keywords.contains(&crate::card::Keyword::Convoke)
            || self.spell_granted_convoke(p, &card);
        let has_improvise = card.definition.keywords.contains(&crate::card::Keyword::Improvise);
        // CR 701.67 — waterbend helpers ride the same `convoke_creatures` slot;
        // any untapped artifact or creature you control may tap to pay {1} of
        // the waterbend sub-cost. Count is clamped to the waterbend amount below.
        let has_waterbend = waterbend.is_some();
        if !convoke_creatures.is_empty() && !has_convoke && !has_improvise && !has_waterbend {
            self.players[p].hand.push(card);
            return Err(GameError::SorcerySpeedOnly); // reuse: spell can't tap helpers
        }
        if let Some(amt) = waterbend
            && convoke_creatures.len() > amt as usize
        {
            self.players[p].hand.push(card);
            return Err(GameError::SorcerySpeedOnly); // reuse: too many waterbend helpers
        }
        for cid in convoke_creatures {
            let bad = !self.battlefield.iter().any(|c| {
                c.id == *cid
                    && c.controller == p
                    && !c.tapped
                    && ((has_convoke && c.definition.is_creature())
                        || (has_improvise && c.definition.is_artifact())
                        || (has_waterbend
                            && (c.definition.is_creature() || c.definition.is_artifact())))
            });
            if bad {
                self.players[p].hand.push(card);
                return Err(GameError::CardNotOnBattlefield(*cid));
            }
        }

        // Validate delve cards up-front (CR 702.66): the spell must have
        // Keyword::Delve and every listed card must currently sit in the
        // caster's graveyard. The cards aren't exiled here — only after the
        // reduced cost is paid — so a rejected cast leaves them in place.
        if !delve_cards.is_empty()
            && !card.definition.keywords.contains(&crate::card::Keyword::Delve)
            && !self.controller_grants_spells_delve(p)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SorcerySpeedOnly); // reuse: spell doesn't have delve
        }
        for cid in delve_cards {
            if !self.players[p].graveyard.iter().any(|c| c.id == *cid) {
                self.players[p].hand.push(card);
                return Err(GameError::CardNotInGraveyard(*cid));
            }
        }

        // Timing: sorcery-speed requires empty stack + main phase + active player priority.
        // Instant-speed (Instant type or Flash) may be cast whenever you have priority.
        // Teferi, Time Raveler's +1 sets `sorceries_as_flash` on its
        // controller — those casters can ignore the sorcery-timing gate
        // until their next turn (when do_untap clears the flag).
        // Teferi's static (`OpponentsSorceryTimingOnly`) flips the rule for
        // opponents: even instants must wait until their main phase.
        // Sigarda's Aid — a battlefield static can grant flash timing to
        // matching spells (Auras + Equipment). Serpent of the Pass — a
        // card-intrinsic `SelfFlashIf` condition on the spell being cast.
        let flash_granted = self.flash_granted_for(p, &card);
        let must_be_sorcery_speed = !(card.definition.is_instant_speed() || flash_granted)
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed
            && !self.can_cast_sorcery_speed(p)
            && !self.players[p].sorceries_as_flash
        {
            self.players[p].hand.push(card);
            return Err(GameError::SorcerySpeedOnly);
        }

        // Validate that the chosen target is legally targetable. Also
        // enforces CR 115.5 — the spell being cast cannot target itself —
        // by threading the cast card's id as `source`.
        if let Some(ref tgt) = target
            && let Err(e) = self.check_target_legality_with_source(tgt, p, Some(card_id))
        {
            self.players[p].hand.push(card);
            return Err(e);
        }
        // Same legality check for each additional target slot (hexproof,
        // shroud, protection, Leyline-of-Sanctity, CR 115.5 self-target).
        for tgt in &additional_targets {
            if let Err(e) = self.check_target_legality_with_source(tgt, p, Some(card_id)) {
                self.players[p].hand.push(card);
                return Err(e);
            }
        }

        // CR 601.2c — an opponent's Flagbearer must be chosen if any declared
        // slot could take it (Standard Bearer).
        {
            let chosen: Vec<Target> =
                target.iter().cloned().chain(additional_targets.iter().cloned()).collect();
            let slots: Vec<Option<crate::card::SelectionRequirement>> = (0..chosen.len())
                .map(|i| {
                    card.definition
                        .effect
                        .target_filter_for_slot_in_mode_kicked(i as u8, mode, kicked)
                        .cloned()
                })
                .collect();
            if self.flagbearer_violation(p, &chosen, &slots) {
                self.players[p].hand.push(card);
                return Err(GameError::InvalidTarget);
            }
        }

        // CR 115.3 — within a *single* multi-target instance ("up to N / any
        // number of / N target …"), the same object can't be chosen twice.
        // Separate "target" clauses (a Seq of single-target effects) may share
        // a target, so this only fires for the divide/support-style effects.
        if let Some(n) = card.definition.effect.distinct_target_count(mode) {
            let mut chosen: Vec<&Target> = Vec::with_capacity(1 + additional_targets.len());
            if let Some(t) = target.as_ref() {
                chosen.push(t);
            }
            chosen.extend(additional_targets.iter());
            chosen.truncate(n as usize);
            for i in 0..chosen.len() {
                for j in (i + 1)..chosen.len() {
                    if chosen[i] == chosen[j] {
                        self.players[p].hand.push(card);
                        return Err(GameError::DuplicateTarget);
                    }
                }
            }
        }

        // CR 702.16: Protection from [color] prevents targeting by spells
        // of that color. Check the spell's colors against the target's
        // protection keywords.
        if let Some(Target::Permanent(cid)) = target
            && let Some(target_card) = self.battlefield_find(cid)
            && target_card.controller != p
        {
            let spell_colors = card.definition.cost.colors();
            // Read computed keywords so granted protection (Mother of Runes,
            // Gods Willing) is honored, not just printed protection.
            let kws = self
                .computed_permanent(cid)
                .map(|cp| cp.keywords.to_vec())
                .unwrap_or_else(|| target_card.definition.keywords.clone());
            for kw in &kws {
                if let Keyword::Protection(prot_color) = kw
                    && spell_colors.contains(prot_color)
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // CR 702.16h — protection from colored spells (Emrakul).
                if matches!(kw, Keyword::ProtectionFromColoredSpells)
                    && !spell_colors.is_empty()
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // Protection from ALL spells (Emrakul, the World Anew), or from
                // everything (Hexdrinker level 8+, Progenitus).
                if matches!(kw, Keyword::ProtectionFromSpells | Keyword::ProtectionFromEverything) {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // CR 702.16 — protection from instants (Hexdrinker level 3-7):
                // an instant spell can't target it.
                if matches!(kw, Keyword::ProtectionFromInstants)
                    && card.definition.card_types.contains(&CardType::Instant)
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // Lurker — "can't be the target of spells unless it attacked
                // or blocked this turn". Abilities still reach it.
                if matches!(kw, Keyword::CantBeTargetedBySpellsUnlessAttackedOrBlocked)
                    && self
                        .battlefield_find(cid)
                        .is_some_and(|c| !c.attacked_this_turn && !c.blocked_this_turn)
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // "Creatures can't be the targets of spells" (Dense Foliage) —
                // no spell may target it; abilities are unaffected.
                if matches!(kw, Keyword::CantBeTargetedBySpells) {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // "Can't be the target of Aura spells" (Bartel Runeaxe,
                // Tetsuo Umezawa) — narrower than protection: only Auras bounce.
                if matches!(kw, Keyword::CantBeTargetedByAuras)
                    && card
                        .definition
                        .subtypes
                        .enchantment_subtypes
                        .contains(&crate::card::EnchantmentSubtype::Aura)
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // CR 702.16 — protection from a spell subtype (Kitsune
                // Riftwalker's "protection from Arcane").
                if let Keyword::ProtectionFromSpellSubtype(sub) = kw
                    && card.definition.subtypes.spell_subtypes.contains(sub)
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // CR 702.16 — protection from each mana value other than N
                // (Haktos): can't be targeted by a spell whose mana value
                // isn't N.
                if let Keyword::ProtectionFromManaValueExcept(n) = kw
                    && card.definition.cost.cmc() != *n
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // CR 702.16 — protection from each mana value of a parity
                // (Lavabrink Venturer): can't be targeted by a spell whose
                // mana value matches the chosen odd/even quality.
                if let Keyword::ProtectionFromManaValueParity { odd } = kw
                    && (card.definition.cost.cmc() % 2 == 1) == *odd
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // CR 702.16 — protection from multicolored: can't be targeted
                // by a spell that is two or more colors.
                if matches!(kw, Keyword::ProtectionFromMulticolored)
                    && spell_colors.len() >= 2
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // CR 702.16 — protection from monocolored: can't be targeted by
                // a spell that is exactly one color.
                if matches!(kw, Keyword::ProtectionFromMonocolored)
                    && spell_colors.len() == 1
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // CR 702.16b — protection from a *filtered* quality
                // (Empty-Shrine Kannushi, Pledge of Loyalty): the spell is
                // still in transient ownership, so match it card-side.
                if let Keyword::ProtectionFromMatching(f) = kw
                    && self.evaluate_requirement_on_card(f, &card, target_card.controller)
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // CR 702.16j — protection from a card type (Serra's Emissary
                // grant): can't be targeted by a spell of that type.
                if let Keyword::ProtectionFromCardType(t) = kw
                    && card.definition.card_types.contains(t)
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
                // "Can't be targeted by nongreen spells opponents control"
                // (Thrun): an opponent's spell that shares none of the listed
                // colors can't target this. Own spells are unaffected.
                if let Keyword::HexproofExceptColors(colors) = kw
                    && self.battlefield_find(cid).is_some_and(|tc| tc.controller != p)
                    && !colors.iter().any(|c| spell_colors.contains(c))
                {
                    self.players[p].hand.push(card);
                    return Err(GameError::TargetHasProtection(cid));
                }
            }
        }

        // CR 702.11e — hexproof from [color]: this object can't be targeted
        // by opponents' spells of that color. Covers both printed
        // `Keyword::HexproofFromColor` and the turn-scoped controller grant
        // (Veil of Summer's "you and permanents you control gain hexproof
        // from blue and black"). Applies to permanent and player targets.
        let spell_colors = card.definition.cost.colors();
        let hexproof_violation = match target {
            Some(Target::Permanent(cid)) => self
                .battlefield_find(cid)
                .filter(|tc| tc.controller != p)
                .is_some_and(|tc| {
                    let controller = tc.controller;
                    let printed = self
                        .computed_permanent(cid)
                        .map(|cp| cp.keywords.to_vec())
                        .unwrap_or_else(|| tc.definition.keywords.clone())
                        .iter()
                        .any(|kw| match kw {
                            Keyword::HexproofFromColor(c) => spell_colors.contains(c),
                            // CR 702.11f — exactly one color on the spell.
                            Keyword::HexproofFromMonocolored => spell_colors.len() == 1,
                            Keyword::HexproofFromMulticolored => spell_colors.len() >= 2,
                            _ => false,
                        });
                    printed
                        || self.players[controller]
                            .hexproof_from_colors_this_turn
                            .iter()
                            .any(|c| spell_colors.contains(c))
                }),
            Some(Target::Player(tp)) => {
                tp != p
                    && self.players[tp]
                        .hexproof_from_colors_this_turn
                        .iter()
                        .any(|c| spell_colors.contains(c))
            }
            None => false,
        };
        if hexproof_violation {
            self.players[p].hand.push(card);
            return Err(GameError::TargetHasHexproof(crate::card::CardId(0)));
        }

        // CR 702.16j — a player with protection from a card type (Serra's
        // Emissary) can't be targeted by a spell of that type.
        if let Some(Target::Player(tp)) = target
            && tp != p
            && self
                .player_protection_card_types(tp)
                .iter()
                .any(|t| card.definition.card_types.contains(t))
        {
            self.players[p].hand.push(card);
            return Err(GameError::TargetHasProtection(crate::card::CardId(0)));
        }

        // Enforce the spell's target selection requirement (e.g. Terror's
        // "non-black, non-artifact creature"): if the effect binds a filter to
        // slot N and the chosen target doesn't match, reject the cast.
        // For modal cards (`ChooseMode`), only look at the chosen mode's
        // filter — Drown in the Loch's mode 0 (counter spell) and mode 1
        // (destroy creature) have incompatible filters, and the legacy
        // "first match across all modes" path picked mode 0's `IsSpellOnStack`
        // even when the caster picked mode 1.
        // Multi-target spells (Snow Day, Render Speechless, Crackle with
        // Power) thread additional slots through `additional_targets`.
        // CR 702.165 — a promised Gift validates (and later resolves) against
        // its enhanced `gifted_effect`, whose target filter can be broader than
        // the printed one (Into the Flood Maw: creature → nonland permanent).
        // Compute the per-slot violation up front (releasing the borrow on
        // `card`) so a rejected cast can move the card back to hand.
        // Cross-slot filters (Barrin's Spite's "controlled by the same player")
        // read the whole chosen slot vector, not just their own target.
        self.target_slots_scratch = std::iter::once(target.clone())
            .chain(additional_targets.iter().cloned().map(Some))
            .collect();
        let filter_violation = {
            let target_effect = if card.gift_promised {
                card.definition.gift.as_ref().map(|g| &g.gifted_effect)
            } else {
                None
            }
            .unwrap_or(&card.definition.effect);
            let slot_bad = |slot: u8, tgt: &Target| {
                target_effect
                    .target_filter_for_slot_in_mode_kicked(slot, mode, kicked)
                    .map(|f| f.resolve_x(x_value.unwrap_or(0)))
                    .is_some_and(|filter| {
                        !self.evaluate_requirement_static(&filter, tgt, p, Some(card.id))
                    })
            };
            target.as_ref().is_some_and(|tgt| slot_bad(0, tgt))
                || additional_targets
                    .iter()
                    .enumerate()
                    .any(|(idx, tgt)| slot_bad((idx + 1) as u8, tgt))
        };
        self.target_slots_scratch.clear();
        if filter_violation {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // "If you cast this spell during your main phase, you may [pick an
        // extra target]" (Return to Dust) — extra slots only on the caster's
        // own main phase.
        if card.definition.extra_targets_main_phase_only
            && !additional_targets.is_empty()
            && !(self.active_player_idx == p && self.step.is_main_phase())
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // "Cast only during combat after blockers are declared" (Flash
        // Foliage).
        if card.definition.cast_only_after_blockers
            && !(self.step.is_combat_phase() && self.blockers_declared)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // "Cast this spell only during combat" (Cauldron Dance).
        if card.definition.cast_only_during_combat && !self.step.is_combat_phase() {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // "Cast only during combat before blockers are declared" (Blaze of
        // Glory).
        if card.definition.cast_only_before_blockers
            && (!self.step.is_combat_phase() || self.blockers_declared)
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // "Cast only before attackers are declared" (Master Warcraft) — legal
        // any time up to and including the Declare Attackers step, until an
        // attacker is actually on the board.
        if card.definition.cast_only_before_attackers
            && (!self.attacking.is_empty()
                || !matches!(
                    self.step,
                    crate::TurnStep::Untap
                        | crate::TurnStep::Upkeep
                        | crate::TurnStep::Draw
                        | crate::TurnStep::PreCombatMain
                        | crate::TurnStep::BeginCombat
                        | crate::TurnStep::DeclareAttackers
                ))
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // "You can't cast this spell unless …" (Rakdos, Lord of Riots).
        if let Some(cond) = card.definition.cast_condition.clone() {
            let ctx = crate::game::effects::EffectContext::for_trigger(card.id, p, None, 0);
            if !self.evaluate_predicate(&cond, &ctx) {
                self.players[p].hand.push(card);
                return Err(GameError::SelectionRequirementViolated);
            }
        }

        // CR 601.2b — additional cast costs ("As an additional cost to cast
        // this spell, sacrifice / discard …"). Validate payability up front
        // so an unpayable spell reverts to hand before any mana is spent;
        // the costs themselves are paid after the mana cost succeeds.
        let mut additional_costs = card.definition.additional_cast_cost.clone();
        // CR 702.41b — an entwined cast also pays the non-mana entwine cost.
        if entwine && let Some(ec) = &card.definition.entwine_additional_cost {
            additional_costs.push(ec.clone());
        }
        // CR 702.27 — "Buyback—Sacrifice a land" (Constant Mists).
        if buyback && let Some(bc) = &card.definition.buyback_additional_cost {
            additional_costs.push(bc.clone());
        }
        // CR 702.32b — a paid action kicker ("Kicker—Sacrifice an artifact")
        // is an additional cost of the kicked cast.
        if kicked && let Some(kc) = &card.definition.kicker_action_cost {
            additional_costs.push(kc.clone());
        }
        // "Sacrifice one or more [filter]" (Plumb the Forbidden): the caster
        // picked the count as the cast's X — concretize into the shared
        // SacrificePermanent payment path.
        for c in additional_costs.iter_mut() {
            match c {
                crate::card::AdditionalCastCost::SacrificeAnyNumber { filter } => {
                    *c = crate::card::AdditionalCastCost::SacrificePermanent {
                        filter: filter.clone(),
                        count: x_value.unwrap_or(0),
                    };
                }
                // "Sacrifice all [filter] you control" (Soulblast) — the count
                // is whatever the caster has right now.
                crate::card::AdditionalCastCost::SacrificeAll { filter } => {
                    let count = self
                        .battlefield
                        .iter()
                        .filter(|b| {
                            b.controller == p
                                && self.evaluate_requirement_static(
                                    filter,
                                    &Target::Permanent(b.id),
                                    p,
                                    None,
                                )
                        })
                        .count() as u32;
                    *c = crate::card::AdditionalCastCost::SacrificePermanent {
                        filter: filter.clone(),
                        count,
                    };
                }
                // "Discard X cards" (Sickening Dreams, the Torment Dreams
                // cycle) — X is the cast's chosen X.
                crate::card::AdditionalCastCost::DiscardXFromCost => {
                    *c = crate::card::AdditionalCastCost::Discard {
                        count: x_value.unwrap_or(0),
                        filter: None,
                    };
                }
                crate::card::AdditionalCastCost::DiscardXRandomFromCost => {
                    *c = crate::card::AdditionalCastCost::DiscardRandom {
                        count: x_value.unwrap_or(0),
                    };
                }
                // "Return X [filter] you control" (Infernal Harvest).
                crate::card::AdditionalCastCost::ReturnToHand {
                    filter,
                    count_x: true,
                    ..
                } => {
                    *c = crate::card::AdditionalCastCost::ReturnToHand {
                        filter: filter.clone(),
                        count: x_value.unwrap_or(0),
                        count_x: false,
                    };
                }
                // "Exile X [filter] cards from your graveyard" (Haunting
                // Misery) — X is the cast's chosen X.
                crate::card::AdditionalCastCost::ExileFromGraveyardXFromCost { filter } => {
                    *c = crate::card::AdditionalCastCost::ExileFromGraveyard {
                        filter: filter.clone(),
                        count: x_value.unwrap_or(0),
                    };
                }
                _ => {}
            }
        }
        if !additional_costs.is_empty() && !self.additional_costs_payable(p, &additional_costs) {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pay the cost (substitute X if present, then add any
        // static-ability tax such as Damping Sphere's "{1} more after the
        // first spell each turn").
        // CR 702.103c — Bestow replaces the regular mana cost with the
        // bestow cost; otherwise the printed cost is used.
        let base_cost = if let (Some(d), Some(room)) = (room_door, card.definition.room.as_deref()) {
            // CR 709.5 — each door is cast for its own cost.
            if d == 1 { room.right.cost.clone() } else { room.left.cost.clone() }
        } else {
            match (bestow, card.definition.has_bestow()) {
                (true, Some(bc)) => bc.clone(),
                _ => card.definition.cost.clone(),
            }
        };
        let mut cost = if base_cost.has_x() {
            base_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            base_cost
        };
        // CR 702.32b — fold the optional kicker cost into the total cost.
        if kicked && let Some(kick) = card.definition.has_kicker() {
            cost.symbols.extend(kick.symbols.iter().cloned());
        }
        // …and each chosen "and/or" kicker option (the Volver cycle).
        for i in &card.kicked_options {
            if let Some(k) = card.definition.kicker_options.get(*i as usize) {
                cost.symbols.extend(k.symbols.iter().cloned());
            }
        }
        // CR 702.172 / FIN Tiered — fold each chosen mode's mana cost into the
        // total.
        if !spree_modes.is_empty()
            && let crate::effect::Effect::Spree { modes } | crate::effect::Effect::Tiered { modes } =
                &card.definition.effect
        {
            for &i in &spree_modes {
                if let Some(m) = modes.get(i as usize) {
                    cost.symbols.extend(m.cost.symbols.iter().cloned());
                }
            }
        }
        // CR 702.27b — fold the optional buyback cost into the total cost,
        // less any "buyback costs cost {N} less" the caster controls
        // (Memory Crystal).
        if buyback && let Some(bb) = card.definition.has_buyback() {
            let mut bb = bb.clone();
            let discount: u32 = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p)
                .flat_map(|c| c.definition.static_abilities.iter())
                .filter_map(|sa| match sa.effect {
                    crate::effect::StaticEffect::BuybackCostsLess { amount } => Some(amount),
                    _ => None,
                })
                .sum();
            if discount > 0 {
                bb.reduce_generic(discount);
            }
            cost.symbols.extend(bb.symbols.iter().cloned());
        }
        // CR 702.41b — fold the optional entwine cost into the total cost.
        if entwine && let Some(ec) = card.definition.has_entwine() {
            for s in &ec.symbols {
                cost.symbols.push(*s);
            }
        }
        // CR 701.67 — fold the waterbend additional cost ({N} generic) into the
        // total. Helpers tap to pay it below; leftover comes from real mana.
        if let Some(amt) = waterbend
            && amt > 0
        {
            cost.symbols.push(crate::mana::ManaSymbol::Generic(amt));
        }
        let tax = extra_cost_for_spell(self, p, &card, target.as_ref());
        if tax > 0 {
            cost.symbols.push(crate::mana::ManaSymbol::Generic(tax));
        }
        cost.symbols.extend(colored_spell_tax_for_spell(self, p, &card).symbols);
        if let Some(extra) = self.flash_surcharge_for(p, &card) {
            cost.symbols.extend(extra.symbols.iter().cloned());
        }
        // Strive (CR 702.122) / Fireball — per-extra-target surcharge.
        cost.symbols.extend(strive_cost_for_spell(&card, additional_targets.len()).symbols);
        cost.symbols.extend(or_pay_cost_symbols(self, p, &card));
        // Apply static cost-reduction effects (Killian's "spells that target
        // a creature cost {2} less"). Tax is applied first so reductions
        // never make the spell free of its tax.
        let mut reduction = cost_reduction_for_spell(self, p, &card, target.as_ref());
        // "…costs {1} less for each permanent sacrificed this way" — the
        // announced sacrifice count rides the cast's X (Rottenmouth Viper).
        if card.definition.self_cost_reduction_per_sacrificed {
            reduction = reduction.saturating_add(x_value.unwrap_or(0));
        }
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        cost.reduce_by_cost(&colored_cost_reduction_for_spell(self, p, &card));
        // Colored-aware target-conditional reduction (Brush Off's "{1}{U}
        // less if it targets an instant or sorcery spell") — mandatory per
        // CR 601.2f, removed pip-by-pip.
        if let Some((filter, less)) = &card.definition.self_cost_reduction_cost_if_target
            && let Some(tgt) = target.as_ref()
            && self.evaluate_requirement_static(filter, tgt, p, Some(card.id))
        {
            cost.reduce_by_cost(less);
        }
        // Delve (CR 702.66): each graveyard card to be exiled pays {1} of the
        // generic cost. The reduction is clamped to the generic portion by
        // `reduce_generic`; the cards themselves are exiled only after a
        // successful payment (below).
        if !delve_cards.is_empty() {
            cost.reduce_generic(delve_cards.len() as u32);
        }
        // Trinisphere floor (CR 117.7 / replacement-style): applied after
        // every reduction so a discounted spell still owes the minimum.
        apply_spell_cost_floor(self, &mut cost);
        // Yusri's jackpot — "you may cast spells from your hand this turn
        // without paying their mana costs" zeroes the whole cost.
        if self.players[p].free_spells_from_hand_this_turn {
            cost.symbols.clear();
        }

        // "Pay X life" additional cost — pre-flight before any payment
        // mutation (CR 119.4: paying down to exactly 0 is legal; below is
        // not). The life itself is paid after the mana receipt succeeds.
        let pay_x_life: u32 = if card.definition.additional_cost_pay_x_life {
            x_value.unwrap_or(0)
        } else {
            0
        };
        if self.effective_life(p) < pay_x_life as i32 {
            self.players[p].hand.push(card);
            return Err(GameError::InsufficientLife);
        }

        // Snapshot pristine state before convoke + auto-tap mutate it, so a
        // failed payment can revert both convoke taps and any lands that
        // auto-tap tapped.
        let snapshot = self.snapshot_payment_state(p);

        // Convoke (CR 702.51): tap each chosen creature; each pays {1} OR one
        // mana of a color that creature is. A tapped creature is credited with
        // a colored pip its color set covers and the cost still needs — so a
        // white creature pays a {W} the cost wants — falling back to {1}.
        // Improvise / waterbend helpers (colorless artifacts) always pay {1}.
        for cid in convoke_creatures {
            let colors: Vec<crate::mana::Color> = self
                .computed_permanent(*cid)
                .map(|cp| cp.colors.to_vec())
                .unwrap_or_default();
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == *cid) {
                c.tapped = true;
            }
            match colors
                .iter()
                .copied()
                .find(|c| self.cost_still_needs_color(p, &cost, *c))
            {
                Some(color) => self.players[p].mana_pool.add(color, 1),
                None => self.players[p].mana_pool.add_colorless(1),
            }
        }

        let forced_only = self.players[p].manual_mana;
        // Spell kind gates spend-restricted mana ("spend only to cast …")
        // by the cast card's types — see `CardDefinition::spell_kind`.
        let spell_kind = card.definition.spell_kind();
        // CR 601.2g — float-spend confirmation. If the caster has pre-existing
        // floating mana that the cost could *either* spend *or* avoid (untapped
        // sources can cover it), ask before auto-spending it instead of silently
        // consuming mana they may have been holding. Skipped for convoke/delve
        // casts (their pool already carries cost-reduction mana). Nothing has
        // been tapped yet on this no-convoke path, so we abort by simply
        // returning the card to hand; the cast re-runs from the top on answer.
        if forced_only
            && spend_float_choice.is_none()
            && convoke_creatures.is_empty()
            && delve_cards.is_empty()
            && self.float_spend_is_optional(p, &cost, &spell_kind)
        {
            let float_summary = self.protectable_float(p, &cost).summary();
            let name = card.definition.name;
            self.players[p].hand.push(card);
            // Replay the exact cast variant (kicker / buyback / bestow survive).
            // Convoke / delve are excluded above, so they never reach here.
            let action = if kicked {
                GameAction::CastSpellKicked { card_id, target, additional_targets, mode, x_value }
            } else if buyback {
                GameAction::CastSpellBuyback { card_id, target, additional_targets, mode, x_value }
            } else if entwine {
                GameAction::CastSpellEntwine { card_id, target, additional_targets, mode, x_value }
            } else if bestow {
                GameAction::CastBestow { card_id, target, additional_targets, mode, x_value }
            } else {
                GameAction::CastSpell { card_id, target, additional_targets, mode, x_value }
            };
            self.pending_decision = Some(crate::game::types::PendingDecision {
                decision: crate::decision::Decision::OptionalTrigger {
                    source: card_id,
                    description: format!(
                        "Spend leftover floating mana ({float_summary}) to cast {name}? (No keeps it and taps lands)"
                    ),
                },
                resume: crate::game::types::ResumeContext::ActionFloatConfirm {
                    actor: p,
                    action: Box::new(action),
                },
            });
            return Ok(vec![]);
        }

        let receipt = match self.try_pay_after_snapshot_mode(p, &cost, snapshot, forced_only, &spell_kind, spend_float_choice) {
            Ok(r) => r,
            Err(e) => {
                self.players[p].hand.push(card);
                return Err(e);
            }
        };
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        self.note_cast_payment_riders(&receipt, &card.definition.spell_kind());
        // "Pay X life" additional cost — paid on cast alongside the mana
        // (CR 601.2h); it stays paid if the spell is countered.
        self.pay_life_cost(p, pay_x_life);

        // CR 702.150 — Compleated: remember the life paid to Phyrexian pips so
        // the planeswalker enters with that much less loyalty.
        if receipt.side_effects.life_lost > 0
            && card.definition.keywords.contains(&crate::card::Keyword::Compleated)
        {
            card.compleated_life_paid = receipt.side_effects.life_lost;
        }

        // Delve payment succeeded — exile the chosen graveyard cards now
        // (CR 702.66: they're exiled as part of paying the cost). Bumps the
        // per-turn exile tally so "if cards were exiled this turn" payoffs see
        // them.
        for cid in delve_cards {
            if let Some(exiled) = Self::take_card(&mut self.players[p].graveyard, *cid) {
                self.exile.push(exiled);
                self.players[p].cards_exiled_this_turn += 1;
            }
        }

        // Compute converge: count distinct colors of mana drained from the
        // pool by paying the cost. Convoke pips contribute generic only,
        // so they don't raise this count.
        let converged_value = converge_count(&receipt.pool_before, &self.players[p].mana_pool);
        // Total mana spent — `pool_before.total() - pool_after.total()`.
        // Read by `Value::CastSpellManaSpent` for Increment / Opus payoffs.
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        card.cast_mana_spent_by_color =
            spent_by_color(&receipt.pool_before, &self.players[p].mana_pool);

        // Chorus of the Conclave — "as an additional cost to cast creature
        // spells, you may pay any amount of mana." Offered after the printed
        // cost is paid, capped by what's still floating; the amount rides
        // `pending_etb_counters` to the battlefield.
        if card.definition.is_creature()
            && self.battlefield.iter().any(|c| {
                c.controller == p
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            sa.effect,
                            crate::effect::StaticEffect::CreatureSpellsMayPayExtraForCounters
                        )
                    })
            })
        {
            let floating = self.players[p].mana_pool.total();
            if floating > 0 {
                let extra = match self.decider.decide(&crate::decision::Decision::ChooseAmount {
                    source: card_id,
                    prompt: "Pay extra mana for +1/+1 counters?".to_string(),
                    max: floating,
                }) {
                    crate::decision::DecisionAnswer::Amount(n) => n.min(floating),
                    _ => 0,
                };
                if extra > 0 {
                    self.players[p].mana_pool.spend_generic(extra);
                    card.pending_etb_counters
                        .push((crate::card::CounterType::PlusOnePlusOne, extra));
                }
            }
        }

        let mut auto_events = receipt.auto_events;
        auto_events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: self.pending_cast_face,
        });

        // CR 601.2h — pay the additional cast costs now (during casting), so
        // sacrifice/discard triggers fire before the spell resolves. A
        // sacrifice reports the fodder's power, which becomes the spell's X
        // for "X = the sacrificed creature's power" riders (Tend the Pests).
        let mut sac_x = None;
        if !additional_costs.is_empty() {
            // CR 701.59 — stamp whether the collect-evidence cost will be paid
            // before payment mutates the graveyard (auto-collects when able), so
            // `Predicate::SpellCollectedEvidence` reads it at resolution.
            if additional_costs.iter().any(|c| matches!(c,
                crate::card::AdditionalCastCost::CollectEvidence { amount, .. }
                    if self.graveyard_can_collect_evidence(p, *amount)))
            {
                card.cast_collected_evidence = true;
            }
            let (mut cost_events, power) =
                self.pay_additional_costs(p, &additional_costs, chosen_sacrifices, chosen_discards);
            auto_events.append(&mut cost_events);
            sac_x = power;
            // Carry the cost-sacrifice's stats into the spell's resolution
            // (resolve_effect resets the scratch) so `Value::Sacrificed*`
            // reads them — Nahiri's Sacrifice's "X = its mana value".
            let had_sac_cost = additional_costs.iter().any(|c| {
                matches!(c, crate::card::AdditionalCastCost::SacrificePermanent { .. })
            });
            if let (true, Some(pw), Some(tough), Some(mv)) = (
                had_sac_cost,
                self.sacrificed_power,
                self.sacrificed_toughness,
                self.sacrificed_mana_value,
            ) {
                let sac_card = self.sacrificed_card;
                let def = std::sync::Arc::make_mut(&mut card.definition);
                def.effect = Effect::WithSacrificedPt {
                    power: pw,
                    total_power: self.sacrificed_total_power,
                    toughness: tough,
                    count: self.sacrificed_count,
                    mana_value: mv,
                    card: sac_card,
                    body: Box::new(def.effect.clone()),
                };
            }
        }
        let events = auto_events;
        let final_x = x_value.unwrap_or(0).max(sac_x.unwrap_or(0));

        // CR 701.67 — record that the waterbend additional cost was paid, so
        // "if its additional cost was paid" riders can branch on resolution.
        if waterbend.is_some_and(|a| a > 0) {
            card.cast_via_waterbend = true;
        }
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            final_x,
            converged_value,
            mana_spent,
            true,
        );

        Ok(events)
    }

    /// Is `sac_id` a legal answer to `card_id`'s "sacrifice a permanent"
    /// additional cast cost? True iff the card is in `caster`'s hand, the
    /// card has such a cost, and `caster` controls `sac_id` and it matches
    /// the cost's filter. Used by the `CastSacrifice` resume to vet a
    /// client's chosen sacrifice before re-running the cast.
    pub(crate) fn cast_sacrifice_choice_is_legal(
        &self,
        caster: usize,
        card_id: CardId,
        sac_id: CardId,
    ) -> bool {
        let Some(card) = self.players[caster].hand.iter().find(|c| c.id == card_id) else {
            return false;
        };
        let Some(sac) = self.battlefield_find(sac_id) else {
            return false;
        };
        if sac.controller != caster {
            return false;
        }
        card.definition
            .additional_cast_cost
            .iter()
            .chain(card.definition.kicker_action_cost.iter())
            .any(|cost| {
                let filter = match cost {
                    crate::card::AdditionalCastCost::SacrificePermanent { filter, .. } => filter,
                    crate::card::AdditionalCastCost::SacrificeOrPay { filter, .. } => filter,
                    _ => return false,
                };
                self.evaluate_requirement_static(filter, &Target::Permanent(sac_id), caster, None)
            })
    }

    /// Pick the `count` lowest-power permanents from `candidates` (by id) —
    /// the AutoDecider heuristic for a forced "Sacrifice another …" activation
    /// cost, so the activator keeps their higher-power creatures. Ties keep the
    /// candidates' (battlefield) order via a stable sort.
    pub(crate) fn auto_pick_lowest_power(&self, candidates: &[CardId], count: usize) -> Vec<CardId> {
        let mut ranked: Vec<(CardId, i32)> = candidates
            .iter()
            .filter_map(|id| self.battlefield_find(*id).map(|c| (*id, c.power())))
            .collect();
        ranked.sort_by_key(|(_, pow)| *pow);
        ranked.into_iter().take(count).map(|(id, _)| id).collect()
    }

    /// Pick the `count` highest-power permanents from `candidates` (by id) —
    /// the AutoDecider heuristic for a "Tap another creature: …" cost whose
    /// payoff *scales* with the tapped creature's power (Station's charge add,
    /// CR 702.184a). Ties keep battlefield order via a stable sort.
    pub(crate) fn auto_pick_highest_power(&self, candidates: &[CardId], count: usize) -> Vec<CardId> {
        let mut ranked: Vec<(CardId, i32)> = candidates
            .iter()
            .filter_map(|id| self.battlefield_find(*id).map(|c| (*id, c.power())))
            .collect();
        ranked.sort_by_key(|(_, pow)| std::cmp::Reverse(*pow));
        ranked.into_iter().take(count).map(|(id, _)| id).collect()
    }

    /// Pick the `count` lowest-mana-value cards (by id) from `player`'s
    /// graveyard among `candidates` — the AutoDecider heuristic for an
    /// "Exile N cards from your graveyard" cost, keeping higher-value cards.
    pub(crate) fn auto_pick_lowest_cmc_gy(
        &self,
        player: usize,
        candidates: &[CardId],
        count: usize,
    ) -> Vec<CardId> {
        let mut ranked: Vec<(CardId, u32)> = candidates
            .iter()
            .filter_map(|id| {
                self.players[player]
                    .graveyard
                    .iter()
                    .find(|c| c.id == *id)
                    .map(|c| (*id, c.definition.cost.cmc()))
            })
            .collect();
        ranked.sort_by_key(|(_, cmc)| *cmc);
        ranked.into_iter().take(count).map(|(id, _)| id).collect()
    }

    /// Pair each `CardId` with its display name from `player`'s graveyard, for
    /// a `Decision::ChooseCards` candidate list. Ids not in the graveyard drop.
    pub(crate) fn graveyard_card_names(&self, player: usize, ids: &[CardId]) -> Vec<(CardId, String)> {
        ids.iter()
            .filter_map(|id| {
                self.players[player]
                    .graveyard
                    .iter()
                    .find(|c| c.id == *id)
                    .map(|c| (*id, c.definition.name.to_string()))
            })
            .collect()
    }

    /// CR 601.2b — can every additional cast cost be paid right now? Checked
    /// before mana so an unpayable spell reverts cleanly.
    pub(crate) fn additional_costs_payable(
        &self,
        p: usize,
        costs: &[crate::card::AdditionalCastCost],
    ) -> bool {
        use crate::card::AdditionalCastCost as A;
        costs.iter().all(|c| match c {
            A::SacrificePermanent { filter, count } => {
                let matching = self.battlefield.iter().filter(|c| {
                    c.controller == p
                        && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, None)
                }).count();
                matching >= *count as usize
            }
            // Concretized into `Discard` by `flashback_additional_costs`
            // before it reaches payment; a raw instance means X = 0.
            A::DiscardXFromCost
            | A::DiscardXRandomFromCost
            | A::ExileFromGraveyardXFromCost { .. } => true,
            // "Sacrifice one or more" — zero is a legal choice, so the cost
            // is always payable (the cast pipeline concretizes it into a
            // counted SacrificePermanent before payment).
            A::SacrificeAnyNumber { .. } | A::SacrificeAll { .. } => true,
            A::Discard { count, filter } => {
                self.players[p]
                    .hand
                    .iter()
                    .filter(|c| {
                        filter.as_ref().is_none_or(|f| self.evaluate_requirement_on_card(f, c, p))
                    })
                    .count()
                    >= *count as usize
            }
            A::DiscardRandom { count } => self.players[p].hand.len() >= *count as usize,
            A::ReturnToHand { filter, count, .. } => {
                let matching = self.battlefield.iter().filter(|c| {
                    c.controller == p
                        && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, None)
                }).count();
                matching >= *count as usize
            }
            A::ExileFromGraveyard { filter, count } => {
                self.players[p]
                    .graveyard
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                    .count()
                    >= *count as usize
            }
            // Reveal-or-pay / sacrifice-or-pay / exile-or-pay are always
            // announceable: with no match the pay half is folded into the
            // spell's cost (`extra_cost_for_spell`) and mana payment
            // enforces it.
            A::RevealFromHandOrPay { .. } => true,
            // A mandatory reveal needs a matching card in hand.
            A::RevealFromHand { filter } => self.players[p]
                .hand
                .iter()
                .any(|c| self.evaluate_requirement_on_card(filter, c, p)),
            A::SacrificeOrPay { .. } => true,
            A::ExileFromGraveyardOrPay { .. } => true,
            A::ProcessExile => self
                .exile
                .iter()
                .any(|c| !self.same_team(c.owner, p)),
            A::TapPermanents { filter, count } => {
                let matching = self.battlefield.iter().filter(|c| {
                    c.controller == p
                        && !c.tapped
                        && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, None)
                }).count();
                matching >= *count as usize
            }
            // CR 119.4 — life can be paid only if the total is at least N.
            A::PayLife { amount } => self.players[p].life >= *amount as i32,
            A::ExilePermanent { filter, count } => {
                let matching = self.battlefield.iter().filter(|c| {
                    c.controller == p
                        && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, None)
                }).count();
                matching >= *count as usize
            }
            A::SacrificeOrPayLife { filter, life } => {
                self.players[p].life >= *life as i32
                    || self.battlefield.iter().any(|c| {
                        c.controller == p
                            && self.evaluate_requirement_static(
                                filter, &Target::Permanent(c.id), p, None,
                            )
                    })
            }
            // Optional collect-evidence is always announceable (may skip);
            // a mandatory one needs enough total MV in the graveyard.
            A::CollectEvidence { amount, optional } => {
                *optional || self.graveyard_can_collect_evidence(p, *amount)
            }
            // Forage-or-pay is always announceable: with no forage material the
            // pay half is folded into the cost.
            A::ForageOrPay { .. } => true,
            // Need a creature to point at — one you control or one to reveal.
            A::ChooseOrRevealCreature => {
                self.battlefield.iter().any(|c| c.controller == p && c.definition.is_creature())
                    || self.players[p].hand.iter().any(|c| c.definition.is_creature())
            }
            // Handing an opponent life is always payable.
            A::OpponentGainsLife { .. } => true,
        })
    }

    /// CR 701.59 — can `p`'s graveyard supply cards with total mana value ≥
    /// `amount` (the requirement to collect evidence `amount`)?
    pub(crate) fn graveyard_can_collect_evidence(&self, p: usize, amount: u32) -> bool {
        let total: u32 = self.players[p]
            .graveyard
            .iter()
            .map(|c| c.definition.cost.cmc())
            .sum();
        total >= amount
    }

    /// CR 701.59 — pay collect evidence `amount`: exile the cheapest set of
    /// `p`'s graveyard cards whose total mana value is ≥ `amount` (keeping the
    /// pricier cards) and emit `EvidenceCollected`. Assumes the caller has
    /// already confirmed `graveyard_can_collect_evidence`. Shared by the cast
    /// additional-cost path and the activated-ability cost path.
    pub(crate) fn collect_evidence_from_graveyard(&mut self, p: usize, amount: u32) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let mut gy: Vec<(CardId, u32)> = self.players[p]
            .graveyard
            .iter()
            .map(|c| (c.id, c.definition.cost.cmc()))
            .collect();
        gy.sort_by_key(|&(_, mv)| mv);
        let mut acc = 0u32;
        let mut to_exile = Vec::new();
        for (id, mv) in gy {
            if acc >= amount {
                break;
            }
            acc += mv;
            to_exile.push(id);
        }
        for id in to_exile {
            if let Some(card) = Self::take_card(&mut self.players[p].graveyard, id) {
                self.exile.push(card);
                self.players[p].cards_exiled_this_turn += 1;
                events.push(GameEvent::PermanentExiled { card_id: id });
            }
        }
        events.push(GameEvent::EvidenceCollected { player: p });
        events
    }

    /// CR 701.61 — can `p` forage (exile three graveyard cards or sacrifice a
    /// Food they control)?
    pub(crate) fn can_forage(&self, p: usize) -> bool {
        self.players[p].graveyard.len() >= 3
            || self.battlefield.iter().any(|c| {
                c.controller == p
                    && c.definition
                        .subtypes
                        .artifact_subtypes
                        .contains(&crate::card::ArtifactSubtype::Food)
            })
    }

    /// CR 601.2h — pay each additional cast cost immediately. Returns the
    /// emitted events plus, for a sacrifice, the sacrificed permanent's
    /// power (threaded into the spell's X).
    pub(crate) fn pay_additional_costs(
        &mut self,
        p: usize,
        costs: &[crate::card::AdditionalCastCost],
        // A `wants_ui` caster's explicit picks (from the `CastAdditionalCost`
        // decision), if any. `chosen_sacrifices` is applied to the first
        // SacrificePermanent cost, `chosen_discards` to the first Discard cost;
        // everything else auto-picks. Owned by the caller (taken from the
        // transient stashes up front) so a failed cast can't leak them.
        chosen_sacrifices: Option<Vec<CardId>>,
        chosen_discards: Option<Vec<CardId>>,
    ) -> (Vec<GameEvent>, Option<u32>) {
        use crate::card::AdditionalCastCost as A;
        let mut events = Vec::new();
        let mut sac_power = None;
        let mut chosen_override = chosen_sacrifices;
        let mut discard_override = chosen_discards;
        for cost in costs {
            match cost {
                // Concretized before payment; a raw instance here (flashback
                // path etc.) means count 0 — no-op.
                A::SacrificeAnyNumber { .. }
                | A::SacrificeAll { .. }
                | A::DiscardXFromCost
                | A::DiscardXRandomFromCost
                | A::ExileFromGraveyardXFromCost { .. } => {}
                A::SacrificePermanent { filter, count } => {
                    // Honor the player's explicit pick(s) when present and
                    // valid; auto-pick the `count` cheapest matching permanents
                    // (tokens first, then lowest mana value, then lowest power)
                    // for any remainder. The first sacrifice's power becomes
                    // the spell's X.
                    let chosen: Vec<SacrificeSnapshot> = {
                        let mut picked: Vec<&crate::card::CardInstance> = Vec::new();
                        if let Some(ids) = chosen_override.take() {
                            for id in ids {
                                if picked.len() >= *count as usize {
                                    break;
                                }
                                if let Some(c) = self.battlefield.iter().find(|c| {
                                    c.id == id
                                        && c.controller == p
                                        && self.evaluate_requirement_static(
                                            filter,
                                            &Target::Permanent(c.id),
                                            p,
                                            None,
                                        )
                                }) {
                                    picked.push(c);
                                }
                            }
                        }
                        if picked.len() < *count as usize {
                            let already: std::collections::HashSet<CardId> =
                                picked.iter().map(|c| c.id).collect();
                            let mut cands: Vec<&crate::card::CardInstance> = self
                                .battlefield
                                .iter()
                                .filter(|c| {
                                    c.controller == p
                                        && !already.contains(&c.id)
                                        && self.evaluate_requirement_static(
                                            filter,
                                            &Target::Permanent(c.id),
                                            p,
                                            None,
                                        )
                                })
                                .collect();
                            cands.sort_by_key(|c| {
                                (!c.is_token, c.definition.cost.cmc(), c.power())
                            });
                            let need = *count as usize - picked.len();
                            picked.extend(cands.into_iter().take(need));
                        }
                        picked
                            .iter()
                            .take(*count as usize)
                            .map(|c| {
                                (
                                    c.id,
                                    c.power().max(0) as u32,
                                    c.definition.is_creature(),
                                    c.toughness(),
                                    c.definition.cost.cmc(),
                                    c.definition.is_artifact(),
                                    c.definition.is_vehicle(),
                                    c.definition.cost.colors(),
                                )
                            })
                            .collect()
                    };
                    self.sacrificed_count = chosen.len() as u32;
                    self.sacrificed_total_power =
                        chosen.iter().map(|c| c.1 as i32).sum();
                    for (idx, (id, power, is_creature, tough, mv, is_artifact, is_vehicle, colors)) in
                        chosen.into_iter().enumerate()
                    {
                        if idx == 0 {
                            sac_power = Some(power);
                            // Stamp the resolution scratch so the spell body can
                            // read `Value::Sacrificed{Power,Toughness,ManaValue}`
                            // (Nahiri's Sacrifice's "X = its mana value").
                            self.sacrificed_power = Some(power as i32);
                            self.sacrificed_toughness = Some(tough);
                            self.sacrificed_mana_value = Some(mv);
                            self.sacrificed_was_artifact = Some(is_artifact);
                            self.sacrificed_was_vehicle = Some(is_vehicle);
                            self.sacrificed_colors = Some(colors);
                            self.sacrificed_card = Some(id);
                        }
                        if is_creature {
                            if let Some(c) = self.dying_snapshot(id) {
                                self.died_card_snapshots.insert(id, c);
                            }
                            events.push(GameEvent::CreatureSacrificed { card_id: id, who: p });
                            events.push(GameEvent::CreatureDied { card_id: id });
                        }
                        events.push(GameEvent::PermanentSacrificed { card_id: id, who: p });
                        let mut die = self.remove_to_graveyard_with_triggers(id);
                        events.append(&mut die);
                    }
                }
                // Acceptable Losses — no pick, so no suspend; the discard
                // funnel picks at random.
                A::DiscardRandom { count } => {
                    for _ in 0..*count {
                        let Some(cid) = self.players[p].hand.first().map(|c| c.id) else { break };
                        self.discard_card(p, cid, &mut events);
                    }
                }
                A::Discard { count, filter } => {
                    // Honor the player's explicit discard picks when present and
                    // still in hand; auto-pick the first *matching* cards in hand
                    // for any remainder (bots/tests, or an under-specified
                    // answer). A `filter` (Magmatic Insight's "a land card")
                    // restricts the auto-pick.
                    let matches = |g: &Self, c: &crate::card::CardInstance| {
                        filter.as_ref().is_none_or(|f| g.evaluate_requirement_on_card(f, c, p))
                    };
                    let mut chosen_ids: Vec<CardId> = Vec::new();
                    if let Some(ids) = discard_override.take() {
                        for id in ids {
                            if chosen_ids.len() >= *count as usize {
                                break;
                            }
                            if self.players[p].hand.iter().any(|c| c.id == id) {
                                chosen_ids.push(id);
                            }
                        }
                    }
                    while chosen_ids.len() < *count as usize {
                        let Some(id) = self
                            .players[p]
                            .hand
                            .iter()
                            .find(|c| !chosen_ids.contains(&c.id) && matches(self, c))
                            .map(|c| c.id)
                        else {
                            break;
                        };
                        chosen_ids.push(id);
                    }
                    for id in chosen_ids {
                        self.discard_card(p, id, &mut events);
                    }
                }
                A::ReturnToHand { filter, count, .. } => {
                    // Auto-pick the lowest-impact matches (tapped first, then
                    // lowest mana value) and bounce them to their owners' hands.
                    let mut cands: Vec<&crate::card::CardInstance> = self
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == p
                                && self.evaluate_requirement_static(
                                    filter, &Target::Permanent(c.id), p, None,
                                )
                        })
                        .collect();
                    cands.sort_by_key(|c| (!c.tapped, c.definition.cost.cmc()));
                    let ids: Vec<CardId> =
                        cands.iter().take(*count as usize).map(|c| c.id).collect();
                    for id in ids {
                        let ctx = EffectContext::for_spell(p, None, 0, 0);
                        self.move_card_to(
                            id,
                            &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::OwnerOfMoved),
                            &ctx,
                            &mut events,
                        );
                    }
                }
                A::ExileFromGraveyard { filter, count } => {
                    // Auto-exile the `count` lowest-MV matching cards from the
                    // caster's graveyard; the first one's mana value becomes X.
                    let mut picks: Vec<(crate::card::CardId, u32)> = self.players[p]
                        .graveyard
                        .iter()
                        .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                        .map(|c| (c.id, c.definition.cost.cmc()))
                        .collect();
                    picks.sort_by_key(|(_, mv)| *mv);
                    for (i, (id, mv)) in picks.into_iter().take(*count as usize).enumerate() {
                        if let Some(card) = Self::take_card(&mut self.players[p].graveyard, id) {
                            self.exile.push(card);
                            events.push(GameEvent::PermanentExiled { card_id: id });
                        }
                        if i == 0 {
                            sac_power = Some(mv);
                        }
                    }
                }
                // Knowledge-only when a matching card is in hand; the pay
                // half was already folded into the cost.
                A::RevealFromHandOrPay { .. } => {}
                // The revealed card stays in hand; stamp its power for the
                // body (Titan's Presence). Reveal the biggest match.
                A::RevealFromHand { filter } => {
                    self.revealed_for_cost_power = self.players[p]
                        .hand
                        .iter()
                        .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                        .map(|c| c.definition.power)
                        .max();
                }
                A::ExileFromGraveyardOrPay { filter, count, .. } => {
                    // With enough matching graveyard cards the exile half is
                    // paid (reusing the ExileFromGraveyard machinery);
                    // otherwise the pay half was already folded into the cost.
                    let matches = self.players[p]
                        .graveyard
                        .iter()
                        .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                        .count() as u32;
                    if matches >= *count {
                        let (mut ev, sp) = self.pay_additional_costs(
                            p,
                            &[A::ExileFromGraveyard { filter: filter.clone(), count: *count }],
                            chosen_override.take(),
                            None,
                        );
                        events.append(&mut ev);
                        if sac_power.is_none() {
                            sac_power = sp;
                        }
                    }
                }
                A::SacrificeOrPay { filter, .. } => {
                    // With a matching permanent the sacrifice half is paid
                    // (reusing the SacrificePermanent machinery); otherwise
                    // the pay half was already folded into the cost.
                    let has_match = self.battlefield.iter().any(|c| {
                        c.controller == p
                            && self.evaluate_requirement_static(
                                filter, &Target::Permanent(c.id), p, None,
                            )
                    });
                    if has_match {
                        let (mut ev, sp) = self.pay_additional_costs(
                            p,
                            &[A::SacrificePermanent { filter: filter.clone(), count: 1 }],
                            chosen_override.take(),
                            None,
                        );
                        events.append(&mut ev);
                        if sac_power.is_none() {
                            sac_power = sp;
                        }
                    }
                }
                A::ProcessExile => {
                    // Auto-pick the lowest-MV opponent-owned exile card and
                    // process it into its owner's graveyard.
                    let pick = self
                        .exile
                        .iter()
                        .filter(|c| !self.same_team(c.owner, p))
                        .min_by_key(|c| c.definition.cost.cmc())
                        .map(|c| c.id);
                    if let Some(id) = pick
                        && let Some(card) = Self::take_card(&mut self.exile, id)
                    {
                        self.route_to_graveyard(card, &mut events);
                    }
                }
                A::TapPermanents { filter, count } => {
                    // Auto-tap the lowest-impact untapped matches (non-lands
                    // first to preserve mana, then lowest mana value).
                    let mut cands: Vec<&crate::card::CardInstance> = self
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == p
                                && !c.tapped
                                && self.evaluate_requirement_static(
                                    filter, &Target::Permanent(c.id), p, None,
                                )
                        })
                        .collect();
                    cands.sort_by_key(|c| (c.definition.is_land(), c.definition.cost.cmc()));
                    let ids: Vec<CardId> =
                        cands.iter().take(*count as usize).map(|c| c.id).collect();
                    for id in ids {
                        if let Some(c) = self.battlefield_find_mut(id) {
                            c.tapped = true;
                            events.push(GameEvent::PermanentTapped { card_id: id, actor: None, as_attacker: false });
                        }
                    }
                }
                A::PayLife { amount } => {
                    let applied = self.adjust_life_applied(p, -(*amount as i32));
                    if applied < 0 {
                        events.push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
                    }
                }
                A::ExilePermanent { filter, count } => {
                    // Honor the caster's explicit pick(s) when present; auto-pick
                    // the cheapest matches (tokens first, then lowest MV).
                    let mut picked: Vec<CardId> = Vec::new();
                    if let Some(ids) = chosen_override.take() {
                        for id in ids {
                            if picked.len() >= *count as usize {
                                break;
                            }
                            if self.battlefield.iter().any(|c| {
                                c.id == id
                                    && c.controller == p
                                    && self.evaluate_requirement_static(
                                        filter, &Target::Permanent(c.id), p, None,
                                    )
                            }) {
                                picked.push(id);
                            }
                        }
                    }
                    if picked.len() < *count as usize {
                        let mut cands: Vec<&crate::card::CardInstance> = self
                            .battlefield
                            .iter()
                            .filter(|c| {
                                c.controller == p
                                    && !picked.contains(&c.id)
                                    && self.evaluate_requirement_static(
                                        filter, &Target::Permanent(c.id), p, None,
                                    )
                            })
                            .collect();
                        cands.sort_by_key(|c| (!c.is_token, c.definition.cost.cmc(), c.power()));
                        let need = *count as usize - picked.len();
                        picked.extend(cands.into_iter().take(need).map(|c| c.id));
                    }
                    for id in picked {
                        let ctx = EffectContext::for_spell(p, None, 0, 0);
                        self.move_card_to(id, &crate::effect::ZoneDest::Exile, &ctx, &mut events);
                    }
                }
                A::SacrificeOrPayLife { filter, life } => {
                    // Sacrifice a matching token if any; else pay life when
                    // affordable; else sacrifice the cheapest match.
                    let has_token = self.battlefield.iter().any(|c| {
                        c.controller == p
                            && c.is_token
                            && self.evaluate_requirement_static(
                                filter, &Target::Permanent(c.id), p, None,
                            )
                    });
                    if has_token || self.players[p].life < *life as i32 {
                        let (mut ev, sp) = self.pay_additional_costs(
                            p,
                            &[A::SacrificePermanent { filter: filter.clone(), count: 1 }],
                            chosen_override.take(),
                            None,
                        );
                        events.append(&mut ev);
                        if sac_power.is_none() {
                            sac_power = sp;
                        }
                    } else {
                        let applied = self.adjust_life_applied(p, -(*life as i32));
                        if applied < 0 {
                            events.push(GameEvent::LifeLost {
                                player: p,
                                amount: (-applied) as u32,
                            });
                        }
                    }
                }
                A::ChooseOrRevealCreature => {
                    // Choose the highest-power creature you control, else reveal
                    // the highest-power creature card in hand; its power becomes
                    // the spell's X (read via `Value::XFromCost`). Nothing moves.
                    let on_bf = self
                        .battlefield
                        .iter()
                        .filter(|c| c.controller == p && c.definition.is_creature())
                        .map(|c| c.power().max(0) as u32)
                        .max();
                    let power = on_bf.or_else(|| {
                        self.players[p]
                            .hand
                            .iter()
                            .filter(|c| c.definition.is_creature())
                            .map(|c| c.definition.power.max(0) as u32)
                            .max()
                    });
                    if sac_power.is_none() {
                        sac_power = power;
                    }
                }
                A::ForageOrPay { .. } => {
                    // With forage material, forage; else the pay half was
                    // already folded into the cost.
                    if self.can_forage(p) {
                        let mut forage = self.pay_forage(p);
                        events.append(&mut forage);
                    }
                }
                A::CollectEvidence { amount, .. } => {
                    // Auto-collect when the graveyard can afford it: exile the
                    // cheapest set of cards summing to ≥ `amount` (keeps the
                    // pricier cards). The `cast_collected_evidence` stamp is set
                    // by the caller from the same graveyard-can-afford check.
                    if self.graveyard_can_collect_evidence(p, *amount) {
                        events.append(&mut self.collect_evidence_from_graveyard(p, *amount));
                    }
                }
                A::OpponentGainsLife { amount } => {
                    if let Some(opp) = self.opponents_of(p).first().copied() {
                        let applied = self.adjust_life_applied(opp, *amount as i32);
                        if applied > 0 {
                            events.push(GameEvent::LifeGained {
                                player: opp,
                                amount: applied as u32,
                            });
                        }
                    }
                }
            }
        }
        (events, sac_power)
    }

    /// Common post-cost-payment bookkeeping for the three cast paths
    /// (`cast_spell_with_convoke`, `cast_flashback`, `cast_spell_alternative`).
    ///
    /// 1. Bumps game-wide and per-player `spells_cast_this_turn` (Storm).
    /// 2. Consumes one Chancellor-of-the-Annex first-spell tax charge.
    /// 3. Stamps `StackItem::Spell.uncounterable` if the caster controls a
    ///    Cavern of Souls of the matching type or the card itself is
    ///    uncounterable.
    /// 4. Pushes the spell onto the stack, then pushes its
    ///    `EventKind::SpellCast` + `EventScope::SelfSource` triggers ABOVE
    ///    it so they resolve first (and still fire if the spell is
    ///    countered in response).
    /// 5. Resets priority to the active player so the cast can be responded
    ///    to.
    // The argument list is wide because each cast path's resolution-time
    // context (`x_value`, `converged_value`, `mana_spent`) must reach
    // `finalize_cast` without round-tripping through a struct. The three
    // callers (regular cast, flashback, alternative cast) all hand off
    // the scalars directly from local variables computed during payment.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn finalize_cast(
        &mut self,
        p: usize,
        card: crate::card::CardInstance,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
        from_hand: bool,
    ) {
        let card_id = card.id;
        // "The next [filter] spell you cast this turn can't be countered" is a
        // one-shot: the matching cast consumes its grant.
        if !self.players[p].next_spell_uncounterable.is_empty() {
            let hit = self.players[p]
                .next_spell_uncounterable
                .iter()
                .position(|f| self.evaluate_requirement_on_card(f, &card, p));
            if let Some(i) = hit {
                self.players[p].next_spell_uncounterable.remove(i);
            }
        }
        self.spells_cast_this_turn += 1;
        // Mana Maze reads the turn's most recent cast (CR 601.2 restriction).
        self.last_cast_spell_colors = card.definition.printed_colors();
        self.players[p].spells_cast_this_turn += 1;
        self.players[p].spells_cast_this_game_turn += 1;
        if card.definition.card_types.contains(&crate::card::CardType::Sorcery) {
            self.players[p].sorceries_cast_this_turn += 1;
        }
        // Arboria — "cast a spell … during their last turn".
        self.note_acted_on_own_turn(p);
        for (seat, pl) in self.players.iter_mut().enumerate() {
            if seat != p {
                pl.opponent_cast_spell_since_your_turn = true;
            }
        }
        // Per-turn cast-name log (Grim Reminder's "cast a spell this turn with
        // the same name").
        self.players[p]
            .spell_names_cast_this_turn
            .push(card.definition.name);
        self.players[p].spell_ids_cast_this_turn.push(card.id);
        // "First noncreature spell of a turn" tally (Nullstone Gargoyle). An
        // Adventure/Omen half cast is a noncreature spell regardless of the
        // card's front face.
        if card.casting_alt_half() || !card.definition.is_creature() {
            self.noncreature_spells_cast_this_turn += 1;
        }
        // Per-name lifetime tally — "you've cast another spell named X this
        // game" (Approach of the Second Sun).
        *self
            .players[p]
            .spells_cast_by_name_this_game
            .entry(card.definition.name)
            .or_insert(0) += 1;
        if from_hand {
            self.players[p].spells_cast_from_hand_this_turn += 1;
        }
        // CR 715 / 702.183 — when cast as its Adventure/Omen half the card is an
        // instant/sorcery spell, not a creature spell, so the spell-type
        // tallies (Magecraft / Prowess) read the half's types.
        let alt_types = card.alt_spell_half().map(|h| &h.card_types);
        let is_instant_or_sorcery = match alt_types {
            Some(types) => {
                types.contains(&CardType::Instant) || types.contains(&CardType::Sorcery)
            }
            None => {
                card.definition.card_types.contains(&CardType::Instant)
                    || card.definition.card_types.contains(&CardType::Sorcery)
            }
        };
        // Refine the spell-type tallies. Both gates default to 0 on
        // snapshot back-compat (player.rs `#[serde(default)]`).
        if is_instant_or_sorcery {
            self.players[p].instants_or_sorceries_cast_this_turn += 1;
        }
        if card.definition.cost.colors().len() >= 2 {
            self.players[p].multicolored_spells_cast_this_turn += 1;
        }
        if !card.casting_alt_half() && card.definition.is_creature() {
            self.players[p].creatures_cast_this_turn += 1;
        }
        // Spell-type tallies for the per-turn lock pieces (Deafening Silence,
        // Ethersworn Canonist). Read the cast half's types so an Adventure/Omen
        // instant-or-sorcery half counts as a noncreature spell.
        let cast_types = alt_types.unwrap_or(&card.definition.card_types);
        if !cast_types.contains(&CardType::Creature) {
            self.players[p].noncreature_spells_cast_this_game_turn += 1;
        }
        if !cast_types.contains(&CardType::Artifact) {
            self.players[p].nonartifact_spells_cast_this_game_turn += 1;
        }
        // Veil of Summer gate: note when a player casts a blue or black
        // spell (color read off the printed mana cost). The full profile
        // (colors + cast half's types) backs the Trap alternative costs.
        {
            let colors = card.definition.cost.colors();
            if colors.contains(&crate::mana::Color::Blue)
                || colors.contains(&crate::mana::Color::Black)
            {
                self.players[p].cast_blue_or_black_this_turn = true;
            }
            self.players[p].spell_casts_this_turn.push(crate::game::types::CastProfile {
                colors,
                card_types: cast_types.clone(),
            });
        }
        consume_first_spell_tax(self, p);

        let on_cast_triggers = if card.casting_alt_half() {
            Vec::new()
        } else {
            collect_self_cast_triggers(&card)
        };
        let uncounterable = self.caster_grants_uncounterable_with_x(p, &card, x_value)
            || std::mem::take(&mut self.cast_paid_uncounterable);

        let was_creature_spell = !card.casting_alt_half() && card.definition.is_creature();
        // CR 702.146e — casting a daybound spell while it's neither day nor
        // night makes it day as the spell is put onto the stack.
        let casts_daybound = card.definition.keywords.contains(&Keyword::Daybound);
        // CR 702.40 — Storm: when this spell is cast, copy it for each spell
        // cast before it this turn. `spells_cast_this_turn` already includes
        // this spell (bumped above), so prior spells = count - 1. Capture the
        // bits needed to mint copies before `card` is moved onto the stack.
        // Statics-granted storm ("Instant and sorcery spells you cast have
        // storm" — Prismari, the Inspiration): granted at cast time so the
        // copy count is the true CR 702.40 storm count.
        let granted_storm = (card.definition.is_instant() || card.definition.is_sorcery())
            && self.battlefield.iter().any(|c| {
                c.controller == p
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(sa.effect, crate::effect::StaticEffect::GrantStormToISSpells)
                    })
            });
        let storm_copies = (card
            .definition
            .keywords
            .contains(&Keyword::Storm)
            || granted_storm)
            .then(|| {
                (
                    card.definition.clone(),
                    self.spells_cast_this_turn.saturating_sub(1),
                )
            })
            // CR 702.69 — Gravestorm: copy for each permanent put into a
            // graveyard from the battlefield this turn (counted before this
            // spell resolves, so its own future death isn't included).
            .or_else(|| {
                card.definition
                    .keywords
                    .contains(&Keyword::Gravestorm)
                    .then(|| (card.definition.clone(), self.permanents_to_graveyard_this_turn))
            })
            // "Copy it for each OTHER instant and sorcery spell you've cast
            // this turn" (Show of Confidence). The caster's I/S counter has
            // already been bumped for this cast, so subtract it back out.
            .or_else(|| {
                card.definition
                    .keywords
                    .contains(&Keyword::SpellStorm)
                    .then(|| {
                        (
                            card.definition.clone(),
                            self.players[p]
                                .instants_or_sorceries_cast_this_turn
                                .saturating_sub(1),
                        )
                    })
            })
            // Caster-chosen copy count via the cast's X (Plumb the
            // Forbidden — one copy per additional-cost sacrifice).
            .or_else(|| {
                card.definition
                    .copies_on_cast_x
                    .then(|| (card.definition.clone(), x_value))
            });

        // CR 608.2b — remember whether the primary target is a battlefield
        // permanent right now; resolution re-checks its legality.
        let mut card = card;
        card.cast_target_was_battlefield = matches!(
            &target,
            Some(Target::Permanent(tid)) if self.battlefield_find(*tid).is_some()
        );
        if self.cast_kick_count > 0 {
            card.kicked = true;
            card.kick_count = self.cast_kick_count;
        }
        self.stack.push(StackItem::Spell {
            card: Box::new(card),
            caster: p,
            target: target.clone(),
            additional_targets: additional_targets.clone(),
            mode,
            x_value,
            converged_value,
            mana_spent,
            uncounterable,
        });
        // Push Storm copies above the original (they resolve first, CR 702.40).
        // Each is a token copy that can't be countered, inheriting target/mode.
        if let Some((def, n)) = storm_copies {
            for _ in 0..n {
                let new_id = self.next_id();
                let mut copy_inst = crate::card::CardInstance::new(new_id, def.clone(), p);
                copy_inst.is_token = true;
                copy_inst.cast_target_was_battlefield = matches!(
                    &target,
                    Some(Target::Permanent(tid)) if self.battlefield_find(*tid).is_some()
                );
                self.stack.push(StackItem::Spell {
                    card: Box::new(copy_inst),
                    caster: p,
                    target: target.clone(),
                    additional_targets: additional_targets.clone(),
                    mode,
                    x_value,
                    converged_value,
                    mana_spent: 0,
                    uncounterable: true,
                });
            }
        }
        self.randomize_single_target_on_stack();
        self.push_on_cast_triggers_x(card_id, p, on_cast_triggers, x_value);
        // SpellCast / YourControl triggers (Prowess, Magecraft, Repartee, …)
        // fire *at cast time*, before the spell resolves. The trigger goes
        // on the stack above the spell so it resolves first (and still
        // fires if the spell itself is countered in response). Filters
        // (e.g. CastSpellTargetsMatch) read the just-cast spell's target
        // from the stack while the spell still sits there.
        //
        // Threads `mana_spent` (and X / Converge) into the trigger context
        // so Increment / Opus payoffs reading `Value::CastSpellManaSpent`
        // observe the actual amount paid for *this* spell.
        self.fire_spell_cast_triggers(p, card_id, !was_creature_spell, mana_spent, converged_value);
        // CR 702.146e — a daybound spell cast while neither day nor night
        // makes it day.
        if casts_daybound && self.day_night.is_none() {
            let mut day_evs = Vec::new();
            self.set_day_night(crate::game::types::DayNight::Day, &mut day_evs);
        }
        // CR 700.14 — Expend. Bump the caster's running spell-mana total and
        // dispatch an `Expended` event so "Whenever you expend N" triggers
        // fire on the cost-payment that first reaches their threshold.
        if mana_spent > 0 {
            self.expend_prev_total = self.mana_spent_on_spells_this_turn;
            self.mana_spent_on_spells_this_turn =
                self.mana_spent_on_spells_this_turn.saturating_add(mana_spent);
            let total = self.mana_spent_on_spells_this_turn;
            self.dispatch_triggers_for_events(&[GameEvent::Expended { player: p, total }]);
        }
        // CR 702.62 suspend accelerants (Deep-Sea Kraken): an opponent's cast
        // ticks a time counter off their suspended accelerant cards.
        self.process_suspend_accelerants(p);
        // CR 702.21: Ward triggers on each chosen target permanent the caster
        // doesn't control. Pushed last so Ward sits on top of the caster's
        // own SpellCast triggers (Magecraft, Prowess) — correct APNAP order
        // since the caster is the active player and Ward belongs to a
        // nonactive player. Ward resolves first and may counter the spell
        // unless the caster pays the Ward cost.
        self.push_ward_triggers_for_cast(p, card_id);
        // BecameTarget triggers — fired through the unified dispatcher so
        // APNAP order is correct and the trigger's `EventSpec.filter` is
        // honored. One event per permanent target on the just-pushed
        // spell. Used by SOS Tenured Concocter's "may draw" trigger.
        self.dispatch_became_target_events_for_cast(p, card_id);
        // CR 700.13 — committing a crime by targeting an opponent / their
        // permanents / cards with this spell (Kaervek, Gisa).
        let crime_targets: Vec<Target> = self
            .stack
            .iter()
            .rev()
            .find_map(|si| match si {
                StackItem::Spell { card, target, additional_targets, .. } if card.id == card_id => {
                    Some(target.iter().cloned().chain(additional_targets.iter().cloned()).collect())
                }
                _ => None,
            })
            .unwrap_or_default();
        self.dispatch_crime_for_targets(p, &crime_targets);
        self.give_priority_to_active();
    }

    /// CR 700.13 — does choosing `t` as a target for a spell/ability cast by
    /// `caster` constitute a crime? True when `t` is an opponent, a permanent
    /// an opponent controls, a spell an opponent controls, or any card an
    /// opponent owns (graveyard / hand / library / exile).
    pub(crate) fn target_is_crime(&self, caster: usize, t: &Target) -> bool {
        match t {
            Target::Player(p) => *p != caster && self.players.get(*p).is_some(),
            Target::Permanent(id) => {
                if let Some(c) = self.battlefield_find(*id) {
                    return c.controller != caster;
                }
                for si in &self.stack {
                    if let StackItem::Spell { card, caster: sc, .. } = si
                        && card.id == *id
                    {
                        return *sc != caster;
                    }
                }
                self.find_card_anywhere(*id).is_some_and(|c| c.owner != caster)
            }
        }
    }

    /// CR 700.13 — dispatch a single `CommittedCrime` event for `caster` if any
    /// of `targets` qualifies as a crime. Fires once per spell/ability, not per
    /// qualifying target.
    pub(crate) fn dispatch_crime_for_targets(&mut self, caster: usize, targets: &[Target]) {
        if targets.iter().any(|t| self.target_is_crime(caster, t)) {
            self.players[caster].committed_crime_this_turn = true;
            self.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: caster }]);
        }
    }

    /// Walk the just-pushed `StackItem::Spell` and emit one
    /// `GameEvent::BecameTarget` for every permanent target slot, then
    /// dispatch the events through the unified trigger pipeline.
    pub(crate) fn dispatch_became_target_events_for_cast(
        &mut self,
        caster: usize,
        cast_card_id: CardId,
    ) {
        let (target, additional_targets): (Option<Target>, Vec<Target>) = match self
            .stack
            .iter()
            .rev()
            .find_map(|si| match si {
                StackItem::Spell { card, target, additional_targets, .. }
                    if card.id == cast_card_id =>
                {
                    Some((target.clone(), additional_targets.clone()))
                }
                _ => None,
            }) {
            Some(t) => t,
            None => return,
        };
        let slots: Vec<Target> = target.into_iter().chain(additional_targets).collect();
        if slots.is_empty() {
            return;
        }
        // CR 601.2c — one "chose targets" event for the whole announcement,
        // plus the per-object `BecameTarget`s.
        let mut events = vec![GameEvent::ChoseTargets { chooser: caster, object: cast_card_id }];
        events.extend(slots.into_iter().filter_map(|t| match t {
            Target::Permanent(id) => {
                Some(GameEvent::BecameTarget { target: id, caster, by: Some(cast_card_id) })
            }
            _ => None,
        }));
        self.dispatch_triggers_for_events(&events);
    }

    /// CR 702.21 — push a Ward triggered ability onto the stack for each
    /// target permanent (controlled by another player) that has
    /// `Keyword::Ward(WardCost)`. Each trigger is `Effect::CounterUnless`
    /// aimed at the just-cast spell. At resolution the engine auto-pays
    /// on the spell controller's behalf if affordable; otherwise the
    /// spell is countered.
    ///
    /// Reads slot 0 + every `additional_targets` slot off the just-pushed
    /// `StackItem::Spell` (so this must run after `finalize_cast`'s push).
    /// Trivial Ward variants (e.g. `WardCost::Mana` with an empty/zero
    /// cost) are skipped — a $0 pay is always affordable and the visible
    /// outcome is identical to no Ward at all, so we save the stack churn.
    pub(crate) fn push_ward_triggers_for_cast(&mut self, caster: usize, cast_card_id: CardId) {
        // Locate the just-pushed spell and pull its targets out as owned
        // values — we can't hold an immutable borrow while we push new
        // stack items below.
        let (target, additional_targets): (Option<Target>, Vec<Target>) = match self
            .stack
            .iter()
            .rev()
            .find_map(|si| match si {
                StackItem::Spell { card, target, additional_targets, .. }
                    if card.id == cast_card_id =>
                {
                    Some((target.clone(), additional_targets.clone()))
                }
                _ => None,
            }) {
            Some(t) => t,
            // Spell isn't on the stack (e.g. countered before this hook).
            None => return,
        };

        let all_targets: Vec<Target> = target
            .into_iter()
            .chain(additional_targets)
            .collect();
        self.push_ward_triggers_for_targets(caster, cast_card_id, &all_targets);
    }

    /// Shared core for Ward enforcement: walk `targets`, and for each
    /// permanent target controlled by a player other than `actor` whose
    /// `Keyword::Ward(WardCost)` is non-trivial, push a Ward trigger
    /// above whatever is currently on top of the stack. The trigger's
    /// `target` carries `target_for_trigger` — the spell card-id (for
    /// casts) or the source permanent's id (for activated abilities) —
    /// so `Effect::CounterUnless` can walk the stack for the topmost
    /// matching `Spell` or `Trigger`.
    pub(crate) fn push_ward_triggers_for_targets(
        &mut self,
        actor: usize,
        target_for_trigger: CardId,
        targets: &[Target],
    ) {
        use crate::card::{Keyword, WardCost};
        use crate::effect::Selector;

        for tgt in targets {
            let perm_id = match tgt {
                Target::Permanent(id) => *id,
                _ => continue,
            };
            self.push_first_targeting_counter(perm_id, target_for_trigger);
            let (ward_cost, ward_controller) = match self
                .battlefield
                .iter()
                .find(|c| c.id == perm_id)
            {
                Some(c) if c.controller != actor => {
                    let computed = self.computed_permanent(perm_id);
                    let cost: Option<WardCost> = computed
                        .as_ref()
                        .map(|cp| cp.keywords.as_slice())
                        .unwrap_or(&c.definition.keywords)
                        .iter()
                        .find_map(|k| match k {
                            Keyword::Ward(cost) => Some(cost.clone()),
                            _ => None,
                        });
                    match cost {
                        Some(cc) if !ward_cost_is_trivial(&cc) => (cc, c.controller),
                        _ => continue,
                    }
                }
                _ => continue,
            };

            let effect = Effect::CounterUnless {
                what: Selector::Target(0),
                cost: ward_cost,
            };
            self.stack.push(
                TriggerPush::new(perm_id, ward_controller, effect)
                    .target(Some(Target::Permanent(target_for_trigger)))
                    .build(),
            );
        }
    }

    /// `Keyword::CounterFirstTargetingEachTurn` (the Glasskite cycle, and
    /// Kira's granted copy): the first spell or ability to target `perm_id`
    /// each turn is countered. Rides the Ward push convention — the trigger's
    /// slot 0 carries the targeting spell / ability source. The per-turn flag
    /// reuses `triggered_once_per_turn_used` under a `usize::MAX` slot, which
    /// no printed trigger index can collide with.
    fn push_first_targeting_counter(&mut self, perm_id: CardId, target_for_trigger: CardId) {
        use crate::card::Keyword;
        let key = (perm_id, usize::MAX);
        if self.triggered_once_per_turn_used.contains(&key) {
            return;
        }
        let Some(controller) = self.battlefield_find(perm_id).map(|c| c.controller) else {
            return;
        };
        let has = self
            .computed_permanent(perm_id)
            .map(|cp| cp.keywords.to_vec())
            .unwrap_or_else(|| {
                self.battlefield_find(perm_id)
                    .map(|c| c.definition.keywords.clone())
                    .unwrap_or_default()
            })
            .contains(&Keyword::CounterFirstTargetingEachTurn);
        if !has {
            return;
        }
        self.triggered_once_per_turn_used.insert(key);
        self.stack.push(
            TriggerPush::new(
                perm_id,
                controller,
                Effect::CounterSpellOrAbility { what: crate::effect::Selector::Target(0) },
            )
            .target(Some(Target::Permanent(target_for_trigger)))
            .build(),
        );
    }

    /// CR 702.21 — Ward enforcement on activated-ability targeting. Hooked
    /// into `activate_ability` immediately after the ability is pushed
    /// onto the stack as a `StackItem::Trigger`. The Ward trigger's
    /// `Effect::CounterUnless` walks the stack for the topmost matching
    /// `Trigger` whose `source` is the activating permanent (identifying
    /// the ability to counter).
    pub(crate) fn push_ward_triggers_for_activated_ability(
        &mut self,
        activator: usize,
        ability_source: CardId,
        target: Option<Target>,
    ) {
        let targets: Vec<Target> = target.into_iter().collect();
        self.push_ward_triggers_for_targets(activator, ability_source, &targets);
    }

    /// Push pre-collected `SpellCast`/`SelfSource` triggers from the
    /// just-cast card onto the stack as `Trigger` items, so they resolve
    /// before the spell itself. Caller is responsible for collecting the
    /// effect list before the card moves into the stack item.
    /// Push a card's "when you cast this spell" triggers, carrying the cast's
    /// `x_value` so an on-cast trigger body can read `Value::XFromCost`
    /// (Hydroid Krasis's "gain half X life and draw half X cards").
    /// Grip of Chaos — "Whenever a spell or ability is put onto the stack, if
    /// it has a single target, reselect its target at random." Called right
    /// after a push; a no-op unless a Grip is on the battlefield.
    pub(crate) fn randomize_single_target_on_stack(&mut self) {
        use crate::effect::StaticEffect;
        use rand::seq::IteratorRandom;
        if !self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::RandomizeSingleTargets))
        }) {
            return;
        }
        let Some(top) = self.stack.last() else { return };
        let (effect, controller, source, single) = match top {
            StackItem::Spell { card, caster, target, additional_targets, .. } => (
                card.definition.effect.clone(),
                *caster,
                Some(card.id),
                target.is_some() && additional_targets.is_empty(),
            ),
            StackItem::Trigger { source, controller, effect, target, .. } => {
                ((**effect).clone(), *controller, Some(*source), target.is_some())
            }
        };
        if !single {
            return;
        }
        let choices =
            self.enumerate_legal_targets_with_source(&effect, controller, source);
        let Some(pick) = choices.into_iter().choose(&mut self.rng.draw()) else { return };
        match self.stack.last_mut() {
            Some(StackItem::Spell { target, .. }) | Some(StackItem::Trigger { target, .. }) => {
                *target = Some(pick);
            }
            None => {}
        }
    }

    pub(crate) fn push_on_cast_triggers_x(
        &mut self,
        source: CardId,
        controller: usize,
        triggers: Vec<(Option<crate::card::Predicate>, Effect)>,
        cast_x: u32,
    ) {
        for (filter, effect) in triggers {
            // Evaluate the trigger's filter (Infusion's
            // LifeGainedThisTurnAtLeast, etc.) before pushing. The ctx
            // is the on-cast context where `trigger_source` points to
            // the cast card. If the filter rejects, drop the trigger
            // silently — matches the "won't trigger unless the
            // condition is met" wording.
            if let Some(pred) = &filter {
                let ctx = crate::game::effects::EffectContext {
                    controller,
                    source: Some(source),
                    targets: vec![],
                    trigger_source: Some(crate::game::effects::EntityRef::Card(source)),
                    mode: 0,
                    x_value: cast_x,
                    converged_value: 0,
                    mana_spent: 0,
                    mana_spent_by_color: Vec::new(),
                    source_name: None,
                    cast_from_hand: true,
                    event_amount: 0,
                    kicked: false,
                    kicked_options: Vec::new(),
                    kick_count: 0,
                    bargained: false,
                    cast_via_mayhem: false,
                    cast_via_waterbend: false,
                    cast_collected_evidence: false,
                    entwined: false,
                    spree_modes: Vec::new(),
                };
                if !self.evaluate_predicate(pred, &ctx) {
                    continue;
                }
            }
            let auto_target =
                self.auto_target_for_effect_avoiding(&effect, controller, Some(source));
            // CR 115.1c — maximize an "up to N target" self-cast trigger
            // (Twisted Riddlekeeper's "tap up to two target permanents") by
            // filling slots 1.. with distinct picks, mirroring the ETB path.
            let additional =
                self.auto_extra_targets_for(&effect, source, controller, auto_target.clone());
            self.stack.push(
                TriggerPush::new(source, controller, effect)
                    .target(auto_target)
                    .additional_targets(additional)
                    .x_value(cast_x)
                    // Self-cast trigger: carry the cast card's id so
                    // Effect::CopySpell can find it on the stack.
                    .trigger_source(Some(crate::game::effects::EntityRef::Card(source)))
                    .build(),
            );
        }
    }

    /// Cast a spell from the graveyard using its Flashback cost.
    /// CR 702.146 — Disturb: cast a graveyard card transformed (its back
    /// face goes on the stack) for its disturb cost, sorcery speed. The
    /// back face's graveyard→exile rider is enforced at the graveyard
    /// funnels off the front face's `Keyword::Disturb`.
    pub(crate) fn cast_disturb(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let graveyard_pos = self.players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let card = self.players[p].graveyard[graveyard_pos].clone();
        // Grafdigger's Cage / Soulless Jailer — no casting from graveyards.
        if self.cast_from_zone_blocked(p, &card.definition, crate::card::Zone::Graveyard) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let disturb_cost = card
            .definition
            .keywords
            .iter()
            .find_map(|k| match k {
                Keyword::Disturb(c) => Some(c.clone()),
                _ => None,
            })
            .ok_or(GameError::SorcerySpeedOnly)?;
        if card.definition.back_face.is_none() {
            return Err(GameError::SorcerySpeedOnly);
        }
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        let mut cost = disturb_cost;
        let reduction = cost_reduction_for_spell_zoned(self, p, &card, None, true);
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        cost.reduce_by_cost(&colored_cost_reduction_for_spell(self, p, &card));
        apply_spell_cost_floor(self, &mut cost);
        let snapshot = self.snapshot_payment_state(p);
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_after_snapshot_mode(
            p, &cost, snapshot, forced_only, &card.definition.spell_kind(), None,
        )?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        self.note_cast_payment_riders(&receipt, &card.definition.spell_kind());
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        // Re-locate by id: `graveyard_pos` was captured before payment ran,
        // and payment side effects may have reshuffled the graveyard.
        let mut card = Self::take_card(&mut self.players[p].graveyard, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        self.entered_from_graveyard_this_turn.insert(card_id);
        // Flip to the back face — the transformed spell goes on the stack
        // (CR 702.146a) and resolves as the back-face permanent; presence was
        // checked before payment above.
        let back = card.definition.back_face.as_ref().map(|b| (**b).clone()).unwrap();
        card.front_face = Some(card.definition.clone());
        card.definition = std::sync::Arc::new(back);
        card.transformed = true;
        // The back face is usually a creature (no target); when it's an Aura it
        // needs an enchant target chosen as the spell goes on the stack (CR
        // 601.2c). Resolution re-checks the target's legality (608.2b).
        let events = vec![
            GameEvent::CardLeftGraveyard { player: p, card_id },
            GameEvent::SpellCast { player: p, card_id, face: CastFace::Front },
        ];
        self.finalize_cast(p, card, target, additional_targets, None, 0, 0, mana_spent, false);
        Ok(events)
    }

    pub(crate) fn cast_flashback(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        // CR 601.2g float-spend choice (None until answered).
        let spend_float = self.pending_cast_spend_float.take();

        // The controller's graveyard — or any graveyard, while Shaman's
        // Trance has pooled them for this seat.
        let card = self
            .find_in_playable_graveyard(p, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?
            .clone();
        // Grafdigger's Cage / Soulless Jailer — no casting from graveyards.
        if self.cast_from_zone_blocked(p, &card.definition, crate::card::Zone::Graveyard) {
            return Err(GameError::CardNotInHand(card_id));
        }

        // The card must have Flashback — printed, or granted until end of
        // turn (the SOS "Flashback" instant) — or Jump-start (CR 702.103:
        // cast for its own mana cost, discarding a card as an additional
        // cost; same exile-after tail as flashback).
        let jumpstart = card.effective_flashback().is_none()
            && card.definition.keywords.contains(&Keyword::JumpStart);
        // CR 702.187 — Mayhem: when the card has no flashback/jump-start but a
        // Mayhem cost, it may be cast from the graveyard for that cost only if
        // its owner discarded it this turn. Same exile-after tail as flashback.
        let mayhem = card.effective_flashback().is_none()
            && !jumpstart
            && card.definition.mayhem_cost().is_some();
        // "You may cast this card from your graveyard …" — own cost plus the
        // card's additional-cost riders, and no exile tail.
        let gy_cast = card.effective_flashback().is_none()
            && !jumpstart
            && !mayhem
            && card.definition.keywords.contains(&Keyword::GraveyardCast);
        // Lier — battlefield static grants flashback (= mana cost) to I/S in
        // the graveyard when nothing else applies.
        let lier_cost = (card.effective_flashback().is_none() && !jumpstart && !mayhem && !gy_cast)
            .then(|| self.graveyard_flashback_grant(p, &card))
            .flatten();
        // Conditional graveyard casting (Viral Spawning's Corrupted gate;
        // Undead Sprinter's "if a non-Zombie creature died this turn"). The
        // gate applies to every graveyard-cast flavor, not just flashback.
        if let Some(cond) = &card.definition.flashback_condition {
            let cctx = crate::game::effects::EffectContext::for_spell(p, None, 0, 0);
            if !self.evaluate_predicate(cond, &cctx) {
                return Err(GameError::SorcerySpeedOnly);
            }
        }
        let flashback_cost = match card.effective_flashback() {
            Some(c) => c.clone(),
            None if jumpstart || gy_cast => card.definition.cost.clone(),
            None if mayhem => {
                if !self.players[p].discarded_this_turn.contains(&card_id) {
                    return Err(GameError::SorcerySpeedOnly);
                }
                card.definition.mayhem_cost().unwrap().clone()
            }
            None => match lier_cost {
                Some(c) => c,
                None => return Err(GameError::SorcerySpeedOnly),
            },
        };

        // {X} flashback costs: a hand-paying caster who didn't send an X
        // picks one via `ChooseAmount` (suspend + clean replay — nothing
        // has been paid yet). Mirrors the cast_spell X prompt, including its
        // [`manual_mana`] gate and the reason for it.
        if flashback_cost.has_x() && x_value.is_none() && self.players[p].manual_mana {
            let max = self.max_prompt_x(p, &flashback_cost);
            let source_name = card.definition.name.to_string();
            self.pending_decision = Some(crate::game::types::PendingDecision {
                decision: crate::decision::Decision::ChooseAmount {
                    source: card_id,
                    max,
                    prompt: format!("{source_name}: choose X"),
                },
                resume: crate::game::types::ResumeContext::CastXPick {
                    caster: p,
                    action: Box::new(crate::game::types::GameAction::CastFlashback {
                        card_id,
                        target,
                        additional_targets,
                        mode,
                        x_value: None,
                    }),
                },
            });
            return Ok(vec![]);
        }

        // Timing: instants can be cast at instant speed, others at sorcery
        // speed. Honor Teferi-style opponent restriction.
        // Sigarda's Aid — a battlefield static can grant flash timing to
        // matching spells (Auras + Equipment). Serpent of the Pass — a
        // card-intrinsic `SelfFlashIf` condition on the spell being cast.
        let flash_granted = self.flash_granted_for(p, &card);
        let must_be_sorcery_speed = !(card.definition.is_instant_speed() || flash_granted)
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }

        // CR 702.34a — flashback-only additional costs ("Flashback—Sacrifice
        // a Mountain"; Dread Return's "sacrifice three creatures"). Keyed by
        // card name (the idiom for rare riders that would otherwise bloat
        // every CardDefinition literal). Reject up front if unpayable so no
        // mana is spent on an uncastable flashback.
        let mut flashback_additional =
            flashback_additional_costs(&card.definition, x_value.unwrap_or(0));
        if jumpstart {
            flashback_additional.push(crate::card::AdditionalCastCost::Discard { count: 1, filter: None });
        }
        if !flashback_additional.is_empty() {
            // The flashback card can't fund its own riders (e.g. Resurgent
            // Belief's gy-exile) — lift it out of the graveyard for the check.
            let lifted = Self::take_card(&mut self.players[p].graveyard, card_id);
            let payable = self.additional_costs_payable(p, &flashback_additional);
            if let Some(c) = lifted {
                self.players[p].graveyard.push(c);
            }
            if !payable {
                return Err(GameError::SelectionRequirementViolated);
            }
        }

        // Validate target.
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
            // Ward enforcement happens via push_ward_triggers_for_cast
            // after finalize_cast, not as a synchronous cost payment.
            let _ = tgt; let _ = p;
        }

        // Pay the flashback cost.
        let mut cost = if flashback_cost.has_x() {
            flashback_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            flashback_cost
        };
        // Flashback IS a cast (CR 702.34a), so Killian-style target-aware
        // cost reductions apply the same as for hand casts. Drain
        // generic-only pips after substituting X.
        let reduction = cost_reduction_for_spell_zoned(self, p, &card, target.as_ref(), true);
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        cost.reduce_by_cost(&colored_cost_reduction_for_spell(self, p, &card));
        // Catalyst Stone — flashback-specific shifts, applied after the
        // generic reductions so the tax can't be reduced away.
        let (fb_less, fb_more) = self.flashback_cost_shift(p);
        if fb_more > 0 {
            cost.symbols.push(crate::mana::ManaSymbol::Generic(fb_more));
        }
        if fb_less > 0 {
            cost.reduce_generic(fb_less);
        }
        apply_spell_cost_floor(self, &mut cost);
        // CR 601.2g — float-spend confirmation. Nothing is mutated yet (the
        // card is still in the graveyard; additional costs unpaid), so suspend
        // cleanly and replay the whole flashback on answer.
        // CR 601.2g float-spend confirmation is a *mana payment* question,
        // so it keys on `manual_mana` — the flag that exists for exactly this
        // rule — rather than `wants_ui`, which bot seats also set. Prompting
        // a bot here is the same livelock as the {X} and additional-cost
        // modals: the suspend returns `Ok`, so the probe reports the cast as
        // legal, and the failed replay is rolled back with the decision
        // restored. Latent rather than observed only because the bot stopped
        // floating mana when it stopped pre-tapping its board.
        if spend_float.is_none()
            && self.players[p].manual_mana
            && !cost.symbols.is_empty()
            && self.float_spend_is_optional(p, &cost, &card.definition.spell_kind())
        {
            let float_summary = self.protectable_float(p, &cost).summary();
            let name = card.definition.name;
            self.pending_decision = Some(crate::game::types::PendingDecision {
                decision: crate::decision::Decision::OptionalTrigger {
                    source: card_id,
                    description: format!(
                        "Spend leftover floating mana ({float_summary}) to flashback {name}? (No keeps it and taps lands)"
                    ),
                },
                resume: crate::game::types::ResumeContext::ActionFloatConfirm {
                    actor: p,
                    action: Box::new(GameAction::CastFlashback {
                        card_id,
                        target: target.clone(),
                        additional_targets: additional_targets.clone(),
                        mode,
                        x_value,
                    }),
                },
            });
            return Ok(vec![]);
        }
        let forced_only = self.players[p].manual_mana;
        let snapshot = self.snapshot_payment_state(p);
        let receipt = self.try_pay_after_snapshot_mode(
            p, &cost, snapshot, forced_only, &card.definition.spell_kind(), spend_float,
        )?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        self.note_cast_payment_riders(&receipt, &card.definition.spell_kind());
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());

        // Pay the flashback-only additional cost(s) now (CR 601.2h). A
        // sacrifice can drop cards into this player's graveyard, shifting
        // indices; `finalize_flashback_cast` re-locates the card by id.
        let mut cost_events = Vec::new();
        if !flashback_additional.is_empty() {
            // Flashback sac costs (Lava Dart, Dread Return) keep the auto-pick
            // — no interactive choice is wired through the flashback path.
            // Lift the flashback card out of the graveyard around the payment
            // so a gy-exile rider (Resurgent Belief) can't pick the spell
            // itself; `finalize_flashback_cast` re-locates it by id.
            let lifted = Self::take_card(&mut self.players[p].graveyard, card_id);
            let (mut e, _sac_power) = self.pay_additional_costs(p, &flashback_additional, None, None);
            if let Some(c) = lifted {
                self.players[p].graveyard.push(c);
            }
            cost_events.append(&mut e);
        }
        let mut events = self.finalize_flashback_cast(
            p,
            card_id,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            mana_spent,
            mayhem,
            gy_cast,
        )?;
        // Sacrifice/discard events precede the cast events in the log.
        cost_events.append(&mut events);
        Ok(cost_events)
    }

    /// CR 702.180 — cast a graveyard card for its Harmonize cost, optionally
    /// tapping one untapped creature you control to reduce the total cost by
    /// generic mana equal to that creature's power. Same exile-after tail as
    /// flashback. Models the alternative cost directly (no float-spend UI loop,
    /// which the simpler graveyard-cast paths also skip).
    pub(crate) fn cast_harmonize(
        &mut self,
        card_id: CardId,
        tap_creature: Option<CardId>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let graveyard_pos = self.players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let card = self.players[p].graveyard[graveyard_pos].clone();
        if self.cast_from_zone_blocked(p, &card.definition, crate::card::Zone::Graveyard) {
            return Err(GameError::CardNotInHand(card_id));
        }
        let harmonize_cost = card
            .effective_harmonize()
            .ok_or(GameError::SorcerySpeedOnly)?
            .clone();

        // Timing: instants at instant speed, the rest at sorcery speed.
        let must_be_sorcery_speed =
            !card.definition.is_instant_speed() || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }

        // CR 702.180b — validate the chosen creature (untapped, yours, a
        // creature) and read its power for the generic-cost reduction.
        let tap_power = if let Some(cid) = tap_creature {
            let c = self
                .battlefield
                .iter()
                .find(|c| c.id == cid)
                .ok_or(GameError::FlashbackTapInvalid)?;
            if c.tapped || c.controller != p || !c.definition.is_creature() {
                return Err(GameError::FlashbackTapInvalid);
            }
            self.computed_permanent(cid).map(|c| c.power.max(0) as u32).unwrap_or(0)
        } else {
            0
        };

        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
        }

        // Build the total cost: harmonize cost (with X), reduced by the tapped
        // creature's power plus any target-aware cost reductions.
        let mut cost = if harmonize_cost.has_x() {
            harmonize_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            harmonize_cost
        };
        let reduction =
            tap_power + cost_reduction_for_spell_zoned(self, p, &card, target.as_ref(), true);
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        apply_spell_cost_floor(self, &mut cost);

        let forced_only = self.players[p].manual_mana;
        let snapshot = self.snapshot_payment_state(p);
        let receipt = self.try_pay_after_snapshot_mode(
            p, &cost, snapshot, forced_only, &card.definition.spell_kind(), None,
        )?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        self.note_cast_payment_riders(&receipt, &card.definition.spell_kind());

        // CR 702.180b — tap the nominated creature as the cost is paid.
        if let Some(cid) = tap_creature
            && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid)
        {
            c.tapped = true;
        }

        self.finalize_flashback_cast(
            p,
            card_id,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            false,
            false,
        )
    }

    /// Shared tail for `cast_flashback` and `cast_flashback_tap`:
    /// remove the card from its owner's graveyard, mark it
    /// `cast_via_flashback` so the resolver exiles it (CR 702.34d),
    /// emit `CardLeftGraveyard` + `SpellCast{Flashback}`, and thread the
    /// rest through `finalize_cast`. `plain_graveyard_cast` suppresses the
    /// exile rider for a card that merely permits a graveyard cast.
    #[allow(clippy::too_many_arguments)]
    fn finalize_flashback_cast(
        &mut self,
        p: usize,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: u32,
        mana_spent: u32,
        via_mayhem: bool,
        plain_graveyard_cast: bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Re-locate by id at removal time: cost payments run before this and
        // can reshuffle the graveyard, so a stored index would be stale.
        let mut card = self
            .take_from_playable_graveyard(p, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        self.entered_from_graveyard_this_turn.insert(card_id);
        card.cast_via_flashback = !plain_graveyard_cast;
        // CR 702.187 — a Mayhem cast stamps the spell so "if the mayhem cost
        // was paid" riders can branch at resolution (Sandman's Quicksand).
        card.cast_via_mayhem = via_mayhem;
        let events = vec![
            GameEvent::CardLeftGraveyard { player: p, card_id },
            GameEvent::SpellCast {
                player: p,
                card_id,
                face: CastFace::Flashback,
            },
        ];
        self.finalize_cast(p, card, target, additional_targets, mode, x_value, 0, mana_spent, false);
        Ok(events)
    }

    /// Cast a graveyard card with `Keyword::Retrace` (CR 702.81): pay its
    /// normal mana cost plus discard a land card from hand. The spell
    /// returns to the graveyard after resolving (no exile), so a single
    /// land + the card can be recast every turn.
    pub(crate) fn cast_retrace(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let card = self.players[p]
            .graveyard
            .iter()
            .find(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?
            .clone();
        // Grafdigger's Cage / Soulless Jailer — no casting from graveyards.
        if self.cast_from_zone_blocked(p, &card.definition, crate::card::Zone::Graveyard) {
            return Err(GameError::CardNotInHand(card_id));
        }
        if !self.effective_retrace(&card, p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // Sigarda's Aid — a battlefield static can grant flash timing to
        // matching spells (Auras + Equipment). Serpent of the Pass — a
        // card-intrinsic `SelfFlashIf` condition on the spell being cast.
        let flash_granted = self.flash_granted_for(p, &card);
        let must_be_sorcery_speed = !(card.definition.is_instant_speed() || flash_granted)
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // Additional cost: a land card in hand to discard. Reject before
        // paying mana if none is available.
        let land_in_hand = self.players[p]
            .hand
            .iter()
            .find(|c| c.definition.is_land())
            .map(|c| c.id)
            .ok_or(GameError::SelectionRequirementViolated)?;
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
        }
        // Pay the printed mana cost (Retrace doesn't change it).
        let mut cost = if card.definition.cost.has_x() {
            card.definition.cost.with_x_value(x_value.unwrap_or(0))
        } else {
            card.definition.cost.clone()
        };
        let reduction = cost_reduction_for_spell_zoned(self, p, &card, target.as_ref(), true);
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        cost.reduce_by_cost(&colored_cost_reduction_for_spell(self, p, &card));
        apply_spell_cost_floor(self, &mut cost);
        let receipt = self.try_pay_with_auto_tap(p, &cost)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        // Discard the land as the additional cost (routes through the
        // central discard path so discard-matters triggers fire).
        let mut events = Vec::new();
        self.discard_card(p, land_in_hand, &mut events);

        // Lift the card from the graveyard and cast it normally — no
        // `cast_via_flashback`, so it returns to the graveyard on
        // resolution and can be retraced again.
        // Re-locate by id: payment + the land discard above ran after the
        // initial scan and can reshuffle the graveyard.
        let card = Self::take_card(&mut self.players[p].graveyard, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        events.push(GameEvent::CardLeftGraveyard { player: p, card_id });
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(
            p, card, target, additional_targets, mode, x_value.unwrap_or(0), 0, mana_spent,
            false,
        );
        Ok(events)
    }

    /// Cast a graveyard card with `Keyword::Escape` (CR 702.139): pay its
    /// escape mana cost plus exiling `exile_cards` (exactly N other cards
    /// from the caster's graveyard). Instants/sorceries resolve back to
    /// the graveyard (so they can be escaped again); permanents enter the
    /// battlefield normally.
    pub(crate) fn cast_escape(
        &mut self,
        card_id: CardId,
        exile_cards: &[CardId],
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let graveyard_pos = self.players[p]
            .graveyard
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let card = self.players[p].graveyard[graveyard_pos].clone();
        let (escape_cost, exile_count) = self
            .effective_escape(&card, p)
            .ok_or(GameError::SorcerySpeedOnly)?;
        // Sigarda's Aid — a battlefield static can grant flash timing to
        // matching spells (Auras + Equipment). Serpent of the Pass — a
        // card-intrinsic `SelfFlashIf` condition on the spell being cast.
        let flash_granted = self.flash_granted_for(p, &card);
        let must_be_sorcery_speed = !(card.definition.is_instant_speed() || flash_granted)
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // The exile set must be exactly N distinct *other* graveyard cards.
        if exile_cards.len() as u32 != exile_count {
            return Err(GameError::SelectionRequirementViolated);
        }
        for cid in exile_cards {
            if *cid == card_id
                || !self.players[p].graveyard.iter().any(|c| c.id == *cid)
            {
                return Err(GameError::SelectionRequirementViolated);
            }
        }
        let mut seen = exile_cards.to_vec();
        seen.sort_unstable();
        seen.dedup();
        if seen.len() != exile_cards.len() {
            return Err(GameError::SelectionRequirementViolated);
        }
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
        }
        // Pay the escape mana cost (with X substitution + reductions).
        let mut cost = if escape_cost.has_x() {
            escape_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            escape_cost
        };
        let reduction = cost_reduction_for_spell_zoned(self, p, &card, target.as_ref(), true);
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        cost.reduce_by_cost(&colored_cost_reduction_for_spell(self, p, &card));
        apply_spell_cost_floor(self, &mut cost);
        let forced_only = self.players[p].manual_mana;
        let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        // Pay the additional cost: exile the chosen graveyard cards.
        let mut events = Vec::new();
        for cid in exile_cards {
            if let Some(exiled) = Self::take_card(&mut self.players[p].graveyard, *cid) {
                events.push(GameEvent::CardLeftGraveyard { player: p, card_id: *cid });
                self.exile.push(exiled);
            }
        }
        // Lift the escaping card from the graveyard and cast it normally —
        // no `cast_via_flashback`, so an instant/sorcery returns to the
        // graveyard on resolution and can be escaped again.
        let mut card = Self::take_card(&mut self.players[p].graveyard, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Stamp the escape-cast flag so the "sacrifice unless it escaped"
        // ETB rider on Kroxa/Uro sees this entered via Escape.
        card.cast_from_escape = true;
        self.entered_from_graveyard_this_turn.insert(card_id);
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        events.push(GameEvent::CardLeftGraveyard { player: p, card_id });
        events.push(GameEvent::SpellCast { player: p, card_id, face: CastFace::Front });
        self.finalize_cast(
            p, card, target, additional_targets, mode, x_value.unwrap_or(0), 0, mana_spent,
            false,
        );
        Ok(events)
    }

    /// Cast a graveyard card with `Keyword::FlashbackTap(N)` by tapping
    /// `tap_creatures` (must be exactly N untapped creatures the caster
    /// controls). The spell costs no mana — the tap is the entire
    /// flashback cost. Routes the resolved card to exile via
    /// `cast_via_flashback` (CR 702.34d). Used by Group Project.
    pub(crate) fn cast_flashback_tap(
        &mut self,
        card_id: CardId,
        tap_creatures: &[CardId],
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let card = self
            .find_in_playable_graveyard(p, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?
            .clone();
        let required_taps = card
            .definition
            .has_flashback_tap()
            .ok_or(GameError::FlashbackTapInvalid)?;
        // Sigarda's Aid — a battlefield static can grant flash timing to
        // matching spells (Auras + Equipment). Serpent of the Pass — a
        // card-intrinsic `SelfFlashIf` condition on the spell being cast.
        let flash_granted = self.flash_granted_for(p, &card);
        let must_be_sorcery_speed = !(card.definition.is_instant_speed() || flash_granted)
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if tap_creatures.len() as u32 != required_taps {
            return Err(GameError::FlashbackTapInvalid);
        }
        // Validate every creature in `tap_creatures` is currently untapped,
        // controlled by the caster, a creature, and matches the keyword's
        // filter if it carries one ("an untapped white creature").
        let tap_filter = card.definition.flashback_tap_filter().cloned();
        for cid in tap_creatures {
            let c = self
                .battlefield
                .iter()
                .find(|c| c.id == *cid)
                .ok_or(GameError::FlashbackTapInvalid)?;
            if c.tapped || c.controller != p || !c.definition.is_creature() {
                return Err(GameError::FlashbackTapInvalid);
            }
            if let Some(f) = &tap_filter
                && !self.evaluate_requirement_static(f, &Target::Permanent(*cid), p, Some(card_id))
            {
                return Err(GameError::FlashbackTapInvalid);
            }
        }
        if let Some(ref tgt) = target {
            self.check_target_legality(tgt, p)?;
        }
        // Pay the tap cost: tap every nominated creature.
        for cid in tap_creatures {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == *cid) {
                c.tapped = true;
            }
        }
        self.finalize_flashback_cast(
            p,
            card_id,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            false,
            false,
        )
    }

    /// Cast a spell from an arbitrary zone (graveyard or exile) without
    /// paying any mana cost. The card is lifted out of `source_zone`,
    /// stamped with `cast_via_flashback = true` when `exile_after` is
    /// set (so the resolver routes it to exile), and threaded through
    /// `finalize_cast` with `mana_spent = 0` / `converged_value = 0`.
    /// No timing / target / cost / alt-cost checks run — the caller is
    /// responsible for any timing window enforcement at the call site.
    ///
    /// Used by:
    /// - `GameAction::CastFromZoneWithoutPaying` (player invokes a
    ///   `may_play_until` permission, e.g. Practiced Scrollsmith's
    ///   exiled card)
    /// - `Effect::CastWithoutPayingImmediate` (immediate resolve-time
    ///   cast — Improvisation Capstone, The Dawning Archaic, Nita)
    /// - `Effect::CastFreeParadigmCopy` (per-main-phase paradigm copy)
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn cast_card_for_free(
        &mut self,
        p: usize,
        card_id: CardId,
        source_zone: crate::card::Zone,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
        exile_after: bool,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Zone;
        // Grafdigger's Cage / Soulless Jailer — no casting from locked
        // zones (free-cast paths included).
        if matches!(source_zone, Zone::Graveyard | Zone::Library | Zone::Exile) {
            let blocked = match source_zone {
                Zone::Exile => self.exile.iter().find(|c| c.id == card_id),
                _ => self.players.iter().find_map(|pl| {
                    pl.graveyard
                        .iter()
                        .chain(pl.library.iter())
                        .find(|c| c.id == card_id)
                }),
            }
            .is_some_and(|c| self.cast_from_zone_blocked(p, &c.definition, source_zone));
            if blocked {
                return Err(GameError::CardNotInHand(card_id));
            }
        }
        // Lift the card out of the named zone. Owner-based zones
        // (graveyard, hand, library) walk all players to locate it.
        let mut card = match source_zone {
            Zone::Exile => Self::take_card(&mut self.exile, card_id)
                .ok_or(GameError::CardNotInHand(card_id))?,
            Zone::Graveyard => {
                let mut found: Option<crate::card::CardInstance> = None;
                for player in self.players.iter_mut() {
                    if let Some(card) = Self::take_card(&mut player.graveyard, card_id) {
                        found = Some(card);
                        break;
                    }
                }
                found.ok_or(GameError::CardNotInHand(card_id))?
            }
            Zone::Hand => {
                // Omniscience-style free cast straight from hand.
                let mut found: Option<crate::card::CardInstance> = None;
                for player in self.players.iter_mut() {
                    if let Some(card) = Self::take_card(&mut player.hand, card_id) {
                        found = Some(card);
                        break;
                    }
                }
                found.ok_or(GameError::CardNotInHand(card_id))?
            }
            Zone::Library => {
                // Cast off the top of a library (Jadzi's magecraft impulse).
                let mut found: Option<crate::card::CardInstance> = None;
                for player in self.players.iter_mut() {
                    if let Some(card) = Self::take_card(&mut player.library, card_id) {
                        found = Some(card);
                        break;
                    }
                }
                found.ok_or(GameError::CardNotInHand(card_id))?
            }
            _ => return Err(GameError::CardNotInHand(card_id)),
        };

        // Clear any outstanding may-play permission — once the card is
        // cast, the grant (and its miracle alt-cost) is consumed.
        card.may_play_until = None;
        card.granted_alt_cast_cost_eot = None;
        // Stamp the cast-zone flag for "cast a spell from exile" payoffs.
        card.cast_from_exile = matches!(source_zone, Zone::Exile);
        card.cast_from_library = matches!(source_zone, Zone::Library);
        // Route to exile on resolve when the granting effect demands it
        // (Nita's "if would go to graveyard, exile instead").
        if exile_after {
            card.cast_via_flashback = true;
        }
        // Bump the "card left graveyard this turn" counter for gy casts.
        if matches!(source_zone, Zone::Graveyard) {
            let owner = card.owner;
            self.players[owner].cards_left_graveyard_this_turn =
                self.players[owner].cards_left_graveyard_this_turn.saturating_add(1);
        }

        let mut events = Vec::new();
        if matches!(source_zone, Zone::Graveyard) {
            events.push(GameEvent::CardLeftGraveyard {
                player: card.owner,
                card_id,
            });
        }
        events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: CastFace::Front,
        });
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            0,
            false,
        );
        Ok(events)
    }

    /// `GameAction::CastFromZoneWithoutPaying` entry point. Validates
    /// that the priority-holding player has an outstanding
    /// `may_play_until` permission on `card_id`, that the permission
    /// hasn't expired, and that the printed-Oracle timing (sorcery vs
    /// instant) allows casting at this window — then hands off to
    /// `cast_card_for_free`.
    pub(crate) fn cast_from_zone_without_paying(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let zone = self
            .find_card_zone(card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Locate the permission + card definition without holding a long
        // borrow across the cast helper.
        let card_ref = self
            .find_card_anywhere(card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        let is_instant = card_ref.definition.is_instant_speed();
        // A "miracle {N}"-style grant attaches an alternative cast cost to
        // the permission (Lorehold, the Historian). When present, the cast
        // isn't free — the controller pays this cost instead of the card's
        // full mana cost.
        let mut alt_cast_cost = card_ref.granted_alt_cast_cost_eot.clone();
        // "It costs [N] more to cast this way unless the spell targets a
        // permanent matching [filter]" (Mavinda's {8} rider). Evaluated
        // against the chosen targets before payment.
        if let Some((surcharge, filt)) = card_ref.granted_cast_surcharge_eot.clone() {
            let targets_match = target
                .iter()
                .chain(additional_targets.iter())
                .any(|t| self.evaluate_requirement_static(&filt, t, p, None));
            if !targets_match {
                let extra = surcharge.cmc();
                match alt_cast_cost.as_mut() {
                    Some(c) => c.symbols.push(crate::mana::generic(extra)),
                    None => alt_cast_cost = Some(surcharge),
                }
            }
        }
        // Two ways to invoke a free cast: a per-card `may_play_until`
        // permission (Discovery / Paradigm / etc.), or an Omniscience-style
        // standing static letting the controller free-cast their own hand
        // spells. The latter doesn't exile the spell afterwards (it goes
        // wherever it normally would).
        // Aluren grants flash to the free creature cast; track it so the
        // sorcery-speed gate below is relaxed for that path only.
        let mut aluren_flash = false;
        let mut miracle_window = false;
        // Valgavoth — "during your turn, you may play cards exiled with this,
        // paying life equal to the spell's mana value instead of its cost."
        let mut valgavoth_toll = (zone == crate::card::Zone::Exile
            && self.active_player_idx == p
            && card_ref.exiled_with.is_some_and(|src| {
                self.battlefield.iter().any(|c| {
                    c.id == src
                        && c.controller == p
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(
                                sa.effect,
                                crate::effect::StaticEffect::PlayExiledWithSourceForLife
                            )
                        })
                })
            }))
        .then(|| card_ref.definition.cost.cmc());
        // Conspiracy Unraveler — collect evidence N instead of the mana cost.
        let mut evidence_toll = None;
        let exile_after = match card_ref.may_play_until {
            Some(permission) => {
                if permission.player != p {
                    return Err(GameError::CardNotInHand(card_id));
                }
                // A real may-play grant drives this cast; don't also bill the
                // Valgavoth life toll.
                valgavoth_toll = None;
                miracle_window = permission.miracle;
                permission.exile_after
            }
            None => {
                let from_own_hand = zone == crate::card::Zone::Hand
                    && self.players[p].hand.iter().any(|c| c.id == card_id);
                if from_own_hand && self.player_casts_hand_spells_free(p, card_ref) {
                    // Omniscience path — no timing relaxation.
                } else if from_own_hand && self.player_casts_cheap_creature_free(&card_ref.definition) {
                    aluren_flash = true;
                } else if from_own_hand
                    && let Some(n) = self.player_casts_spells_for_evidence(p)
                {
                    evidence_toll = Some(n);
                } else if valgavoth_toll.is_some() {
                    // The life toll below stands in for the mana cost.
                } else {
                    return Err(GameError::CardNotInHand(card_id));
                }
                false
            }
        };
        // Expiry check: EndOfThisTurn => only valid this turn;
        // EndOfControllersNextTurn => one full controller-turn later.
        // Defensive — the cleanup hook also clears expired permissions.
        // CR 702.94e — a Miracle cast happens inside the reveal trigger's
        // window, where the rules permit casting a sorcery; skip the gate.
        let must_be_sorcery_speed = !aluren_flash
            && !miracle_window
            && (!is_instant || self.player_locked_to_sorcery_timing(p));
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // Pay the miracle alt-cost up front, if any. Failure leaves the
        // permission intact so the controller can retry once they have the
        // mana.
        // Warped Space — once each turn, a cast from exile may pay {0}
        // instead of the cost its may-play grant stamped on.
        let waive = zone == crate::card::Zone::Exile
            && alt_cast_cost.as_ref().is_some_and(|c| c.cmc() > 0)
            && !self.players[p].free_exile_cast_used_this_turn
            && self.battlefield.iter().any(|c| {
                c.controller == p
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(sa.effect, crate::effect::StaticEffect::FreeExileCastOncePerTurn)
                    })
            });
        if let Some(life) = valgavoth_toll {
            if self.players[p].life < life as i32 {
                return Err(GameError::InsufficientLife);
            }
            self.pay_life_cost(p, life);
        }
        if waive {
            self.players[p].free_exile_cast_used_this_turn = true;
        } else if let Some(cost) = alt_cast_cost {
            let forced_only = self.players[p].manual_mana;
            let receipt = self.try_pay_with_auto_tap_mode(p, &cost, forced_only)?;
            self.pay_life_cost(p, receipt.side_effects.life_lost);
        }
        let mut events = match evidence_toll {
            Some(n) => self.collect_evidence_from_graveyard(p, n),
            None => Vec::new(),
        };
        events.extend(self.cast_card_for_free(
            p,
            card_id,
            zone,
            target,
            additional_targets,
            mode,
            x_value,
            exile_after,
        )?);
        Ok(events)
    }

    /// Cast a spell using its `alternative_cost` (a "pitch" cost) instead of
    /// its regular mana cost. Pays the alt cost's mana, deducts life, and
    /// exiles the chosen `pitch_card` from hand if the alt cost requires
    /// Cast a commander from your command zone (Phase L).
    ///
    /// Differences vs. `cast_spell`:
    /// * The card is sourced from `players[p].command` instead of
    ///   `players[p].hand`.
    /// * Cost = printed cost + `{2}` × `commander_cast_count[card_id]`
    ///   (the commander tax, CR 903.8).
    /// * On a successful payment the cast count is bumped so the next
    ///   cast pays `{4}` extra, then `{6}`, etc.
    /// * The Phase J zone-change replacement is already registered for
    ///   each seated commander, so when the resulting permanent
    ///   eventually leaves play it gets snagged back into the command
    ///   zone automatically.
    ///
    /// Sorcery-speed / target legality / etc. piggyback off the same
    /// helpers as `cast_spell_with_convoke` to keep behavior aligned.
    /// X / mode / target slots are threaded through verbatim.
    pub(crate) fn cast_from_command_zone(
        &mut self,
        card_id: CardId,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;

        // Locate + remove the commander from the caster's command zone.
        let mut card = Self::take_card(&mut self.players[p].command, card_id)
            .ok_or(GameError::CardNotInHand(card_id))?;
        card.cast_from_hand = false;

        // Sorcery-speed gate (commanders are creatures by definition,
        // which are sorcery-speed unless flash). We rebuild the same
        // gate `cast_spell` uses so timing matches.
        // Sigarda's Aid — a battlefield static can grant flash timing to
        // matching spells (Auras + Equipment). Serpent of the Pass — a
        // card-intrinsic `SelfFlashIf` condition on the spell being cast.
        let flash_granted = self.flash_granted_for(p, &card);
        let must_be_sorcery_speed = !(card.definition.is_instant_speed() || flash_granted)
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed
            && !self.can_cast_sorcery_speed(p)
            && !self.players[p].sorceries_as_flash
        {
            self.players[p].command.push(card);
            return Err(GameError::SorcerySpeedOnly);
        }

        // Target legality (the rare commander with a targeted ETB
        // wants the same hexproof / shroud / Leyline checks).
        if let Some(ref tgt) = target
            && let Err(e) = self.check_target_legality_with_source(tgt, p, Some(card_id))
        {
            self.players[p].command.push(card);
            return Err(e);
        }
        for tgt in &additional_targets {
            if let Err(e) = self.check_target_legality_with_source(tgt, p, Some(card_id)) {
                self.players[p].command.push(card);
                return Err(e);
            }
        }

        // Build the cost: printed + commander tax. The tax is
        // `{2}` × prior casts; it stacks on top of any X / generic
        // tax / cost reduction the spell would normally see.
        let base_cost = card.definition.cost.clone();
        let mut cost = if base_cost.has_x() {
            base_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            base_cost
        };
        let prior = self.commander_cast_count.get(&card_id).copied().unwrap_or(0);
        let commander_tax = prior.saturating_mul(2);
        if commander_tax > 0 {
            cost.symbols
                .push(crate::mana::ManaSymbol::Generic(commander_tax));
        }
        let tax = extra_cost_for_spell(self, p, &card, target.as_ref());
        if tax > 0 {
            cost.symbols
                .push(crate::mana::ManaSymbol::Generic(tax));
        }
        cost.symbols.extend(colored_spell_tax_for_spell(self, p, &card).symbols);
        if let Some(extra) = self.flash_surcharge_for(p, &card) {
            cost.symbols.extend(extra.symbols.iter().cloned());
        }
        cost.symbols.extend(strive_cost_for_spell(&card, additional_targets.len()).symbols);
        cost.symbols.extend(or_pay_cost_symbols(self, p, &card));
        let reduction = cost_reduction_for_spell(self, p, &card, target.as_ref());
        if reduction > 0 {
            cost.reduce_generic(reduction);
        }
        cost.reduce_by_cost(&colored_cost_reduction_for_spell(self, p, &card));
        apply_spell_cost_floor(self, &mut cost);

        // Pay. On failure put the card back in the command zone.
        let forced_only = self.players[p].manual_mana;
        let receipt = match self.try_pay_with_auto_tap_mode(p, &cost, forced_only) {
            Ok(r) => r,
            Err(e) => {
                self.players[p].command.push(card);
                return Err(e);
            }
        };
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let converged_value = converge_count(&receipt.pool_before, &self.players[p].mana_pool);
        let mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());

        // Bump the cast counter on success.
        *self.commander_cast_count.entry(card_id).or_insert(0) += 1;

        let mut auto_events = receipt.auto_events;
        auto_events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: self.pending_cast_face,
        });
        let events = auto_events;

        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            converged_value,
            mana_spent,
            false,
        );

        Ok(events)
    }

    /// one. The spell otherwise behaves identically to a normal cast (goes
    /// onto the stack, resolves later, etc.).
    pub(crate) fn cast_spell_alternative(
        &mut self,
        card_id: CardId,
        pitch_card: Option<CardId>,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: Option<usize>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        // CR 601.2g float-spend choice (None until answered).
        let spend_float = self.pending_cast_spend_float.take();

        if !self.players[p].has_in_hand(card_id) {
            return Err(GameError::CardNotInHand(card_id));
        }
        // Validate the spell actually has an alternative cost; clone it before
        // any mutation so we don't borrow the card twice.
        let alt = self.effective_alternative_cost(p, card_id).ok_or(GameError::NoAlternativeCost)?;

        // Force of Negation–style "you may pay this alt cost only if it's
        // not your turn." Reject the alt cast on the caster's own turn —
        // they can still pay the regular mana cost via `cast_spell`.
        if alt.not_your_turn_only && self.active_player_idx == p {
            return Err(GameError::NoAlternativeCost);
        }

        // Optional cast-time predicate gate. Used by
        // SOS Wilt in the Heat's "{2} less if cards left your graveyard
        // this turn" rider, where the alt cost is only legal under a
        // specific game-state condition. Rejected before any state
        // mutation (no card removal, no mana payment) so callers can
        // retry the cast via the regular cost path.
        if let Some(cond) = &alt.condition {
            let ctx = crate::game::effects::EffectContext {
                controller: p,
                source: Some(card_id),
                targets: vec![],
                trigger_source: None,
                mode: 0,
                x_value: 0,
                converged_value: 0,
                mana_spent: 0,
                mana_spent_by_color: Vec::new(),
                source_name: None,
                cast_from_hand: true,
                event_amount: 0,
                kicked: false,
                kicked_options: Vec::new(),
                kick_count: 0,
                bargained: false,
                cast_via_mayhem: false,
                cast_via_waterbend: false,
                cast_collected_evidence: false,
                    entwined: false,
                    spree_modes: Vec::new(),
            };
            if !self.evaluate_predicate(cond, &ctx) {
                return Err(GameError::NoAlternativeCost);
            }
        }

        // CR 119.4: A player can only pay an amount of life if their
        // life total is greater than or equal to the payment. Pre-flight
        // gate so we reject cleanly rather than driving life negative
        // mid-cast.
        if alt.life_cost > 0 && self.players[p].life < alt.life_cost as i32 {
            return Err(GameError::InsufficientLife);
        }

        // Pre-flight: confirm the caster has enough cards in their
        // graveyard for the `exile_from_graveyard_count` additional
        // cost. Picks are committed AFTER the mana payment succeeds
        // (mirroring `exile_other_filter` on activated abilities) so a
        // failed mana pay rolls back cleanly. The auto-picker takes the
        // lowest-CMC matching cards so higher-value graveyard cards
        // stay put.
        let exile_gy_picks: Vec<CardId> = if alt.exile_from_graveyard_count > 0 {
            let n = alt.exile_from_graveyard_count as usize;
            let mut picks: Vec<(CardId, i32)> = self.players[p]
                .graveyard
                .iter()
                .map(|c| (c.id, c.definition.cost.cmc() as i32))
                .collect();
            picks.sort_by_key(|(_, cmc)| *cmc);
            if picks.len() < n {
                return Err(GameError::SelectionRequirementViolated);
            }
            picks.into_iter().take(n).map(|(cid, _)| cid).collect()
        } else {
            Vec::new()
        };

        // Return-N-permanents-to-hand additional cost (Gush / Daze). Pick
        // the matches up front so a shortfall rejects before any mana is
        // paid; commit the moves after payment succeeds. Auto-picker
        // prefers untapped permanents (lower tempo loss).
        let return_picks: Vec<CardId> = if let Some((filter, count)) = &alt.return_to_hand {
            let n = *count as usize;
            let mut matches: Vec<(CardId, bool)> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == p
                        && self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(c.id),
                            p,
                            None,
                        )
                })
                .map(|c| (c.id, c.tapped))
                .collect();
            matches.sort_by_key(|(_, tapped)| *tapped);
            if matches.len() < n {
                return Err(GameError::SelectionRequirementViolated);
            }
            matches.into_iter().take(n).map(|(cid, _)| cid).collect()
        } else {
            Vec::new()
        };

        // Sacrifice-N-permanents additional cost (Fireblast). Same up-front
        // pick / late commit discipline as `return_picks`.
        let sacrifice_picks: Vec<CardId> = if let Some((filter, count)) = &alt.sacrifice_permanents {
            let n = *count as usize;
            let matches: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == p
                        && self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(c.id),
                            p,
                            None,
                        )
                })
                .map(|c| c.id)
                .collect();
            if matches.len() < n {
                return Err(GameError::SelectionRequirementViolated);
            }
            matches.into_iter().take(n).collect()
        } else {
            Vec::new()
        };

        // Tap-N-creatures additional cost (Orim's Cure). Same up-front pick /
        // late commit discipline; the auto-picker taps the lowest-power
        // untapped matches so the good attackers stay up.
        let tap_picks: Vec<CardId> = if let Some((filter, count)) = &alt.tap_creatures {
            let n = *count as usize;
            let mut matches: Vec<(CardId, i32)> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == p
                        && !c.tapped
                        && self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(c.id),
                            p,
                            None,
                        )
                })
                .map(|c| (c.id, self.computed_permanent(c.id).map_or(0, |cp| cp.power)))
                .collect();
            matches.sort_by_key(|(_, pow)| *pow);
            if matches.len() < n {
                return Err(GameError::SelectionRequirementViolated);
            }
            matches.into_iter().take(n).map(|(cid, _)| cid).collect()
        } else {
            Vec::new()
        };

        // CR 702.119 — Emerge: pick the creature to sacrifice (auto: highest
        // MV for max cost reduction) and record its MV. Rejected up front if
        // the caster controls no matching creature. Sacrificed after payment.
        let (emerge_sac, emerge_reduction): (Option<CardId>, u32) =
            if let Some(filter) = &alt.emerge {
                let best = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == p
                            && c.definition.is_creature()
                            && self.evaluate_requirement_static(
                                filter,
                                &Target::Permanent(c.id),
                                p,
                                None,
                            )
                    })
                    .max_by_key(|c| c.definition.cost.cmc())
                    .map(|c| (c.id, c.definition.cost.cmc()));
                match best {
                    Some((cid, mv)) => (Some(cid), mv),
                    None => return Err(GameError::SelectionRequirementViolated),
                }
            } else {
                (None, 0)
            };

        // CR 702.48 — Offering: pick the creature to sacrifice (auto: highest
        // MV for max reduction) and record its whole mana cost. Rejected up
        // front if the caster controls no matching creature. Sacrificed after
        // payment; the cost reduction is by the full cost, color included.
        let offering_pick: Option<(CardId, crate::mana::ManaCost)> =
            if let Some(filter) = &alt.offering {
                let best = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == p
                            && c.definition.is_creature()
                            && self.evaluate_requirement_static(
                                filter,
                                &Target::Permanent(c.id),
                                p,
                                None,
                            )
                    })
                    .max_by_key(|c| c.definition.cost.cmc())
                    .map(|c| (c.id, c.definition.cost.clone()));
                match best {
                    Some(x) => Some(x),
                    None => return Err(GameError::SelectionRequirementViolated),
                }
            } else {
                None
            };

        // Validate that the pitch card matches the filter (if any).
        if let Some(filter) = &alt.exile_filter {
            let pitch_id = pitch_card.ok_or(GameError::NoAlternativeCost)?;
            // The pitch card must be in hand AND match the filter. The filter
            // typically refers to spell colors (e.g. HasColor(Blue)) so we
            // evaluate it against the card's definition rather than against
            // a battlefield CardInstance.
            let pitch_card_inst = self.players[p]
                .hand
                .iter()
                .find(|c| c.id == pitch_id)
                .ok_or(GameError::InvalidPitchCard(pitch_id))?;
            // The pitch card must not be the spell itself.
            if pitch_id == card_id {
                return Err(GameError::InvalidPitchCard(pitch_id));
            }
            // CR 601.2b — the Shoal cycle's "exile a [color] card with mana
            // value X": the filter's X atoms read the declared X.
            let filter = filter.resolve_x(x_value.unwrap_or(0));
            if !self.evaluate_requirement_on_card(&filter, pitch_card_inst, p) {
                return Err(GameError::InvalidPitchCard(pitch_id));
            }
        }

        // CR 601.2b — "discard a [filter] card rather than pay this spell's
        // mana cost": pick the discards up front (lowest MV first) so an
        // unpayable alt cost is rejected before anything is spent.
        let mut discard_picks: Vec<CardId> = Vec::new();
        for (filter, n) in &alt.discard_filters {
            for _ in 0..*n {
                let pick = self.players[p]
                    .hand
                    .iter()
                    .filter(|c| {
                        c.id != card_id
                            && !discard_picks.contains(&c.id)
                            && self.evaluate_requirement_on_card(filter, c, p)
                    })
                    .min_by_key(|c| c.definition.cost.cmc())
                    .map(|c| c.id);
                match pick {
                    Some(id) => discard_picks.push(id),
                    None => return Err(GameError::SelectionRequirementViolated),
                }
            }
        }

        // Remove the spell card from hand now (so the pitch card doesn't
        // accidentally collide with it during validation).
        let mut card = self.players[p].remove_from_hand(card_id).unwrap();
        card.cast_from_hand = true;
        card.cast_from_exile = false;
        card.cast_from_library = self.casting_from_library_top == Some(card_id);
        if alt.evoke_sacrifice {
            card.evoked = true;
        }
        if alt.dash {
            card.dashed = true;
        }
        if alt.blitz {
            card.blitzed = true;
        }
        if alt.warp {
            // EOE Warp — stamp the resolving permanent and satisfy Void for
            // the rest of the turn.
            card.warped = true;
            self.players[p].warped_spell_this_turn = true;
        }
        if alt.converted {
            // CR 701.28 — cast converted: the permanent enters transformed.
            card.cast_converted = true;
        }
        if alt.impending > 0 {
            // CR 702.183 — enters with N time counters; stamped now, applied
            // at ETB resolution.
            card.impending_counters = alt.impending;
        }
        if alt.marks_kicked {
            // CR 702.108 — Surge: stamp the spell kicked so "if its surge
            // cost was paid" ETB riders fire via `SpellWasKicked`.
            card.kicked = true;
        }

        // Timing: sorcery-speed unless instant-speed (or the alt cost grants
        // flash — Rout), plus Teferi-style opponent restriction.
        let must_be_sorcery_speed = (!card.definition.is_instant_speed() && !alt.flash)
            || self.player_locked_to_sorcery_timing(p);
        if must_be_sorcery_speed && !self.can_cast_sorcery_speed(p) {
            self.players[p].hand.push(card);
            return Err(GameError::SorcerySpeedOnly);
        }

        // Validate target legality.
        if let Some(ref tgt) = target
            && let Err(e) = self.check_target_legality(tgt, p)
        {
            self.players[p].hand.push(card);
            return Err(e);
        }
        // When the alt cost carries an effect_override, use its target
        // filter instead of the base spell's (kicker-style alt modes
        // change the legal target set). Otherwise, validate against the
        // base spell's filter.
        {
            let effect_for_filter = alt.effect_override.as_ref()
                .unwrap_or(&card.definition.effect);
            if let Some(ref tgt) = target
                && let Some(filter) = effect_for_filter
                    .target_filter_for_slot_in_mode(0, mode)
                && !self.evaluate_requirement_static(filter, tgt, p, Some(card.id))
            {
                self.players[p].hand.push(card);
                return Err(GameError::SelectionRequirementViolated);
            }
        }
        // Alt-cost-specific target filter (e.g. Mystical Dispute's "target
        // must be a blue spell"). Applied on top of the spell's regular
        // target filter, only on the alternative-cast path.
        if let Some(ref tgt) = target
            && let Some(ref alt_filter) = alt.target_filter
            && !self.evaluate_requirement_static(alt_filter, tgt, p, Some(card.id))
        {
            self.players[p].hand.push(card);
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pay the alt mana cost (with X substitution + static-ability tax + Ward).
        let mut mana_cost = if alt.mana_cost.has_x() {
            alt.mana_cost.with_x_value(x_value.unwrap_or(0))
        } else {
            alt.mana_cost.clone()
        };
        let tax = extra_cost_for_spell(self, p, &card, target.as_ref());
        if tax > 0 {
            mana_cost.symbols.push(crate::mana::ManaSymbol::Generic(tax));
        }
        mana_cost.symbols.extend(colored_spell_tax_for_spell(self, p, &card).symbols);
        if let Some(extra) = self.flash_surcharge_for(p, &card) {
            mana_cost.symbols.extend(extra.symbols.iter().cloned());
        }
        mana_cost.symbols.extend(strive_cost_for_spell(&card, additional_targets.len()).symbols);
        mana_cost.symbols.extend(or_pay_cost_symbols(self, p, &card));
        // CR 601.2f: cost reductions apply uniformly across cast paths
        // (hand cast / flashback / alt-cost), and `cost_reduction_for_
        // spell` returns the same delta in each. The alt cost is often
        // {0} for pitch spells (Force of Negation, Mystical Dispute), in
        // which case the reduction simply no-ops (clamps at zero).
        let reduction = cost_reduction_for_spell(self, p, &card, target.as_ref()) + emerge_reduction;
        if reduction > 0 {
            mana_cost.reduce_generic(reduction);
        }
        // Colored-aware target-conditional reduction — applies uniformly
        // across cast paths (CR 601.2f).
        if let Some((filter, less)) = &card.definition.self_cost_reduction_cost_if_target
            && let Some(tgt) = target.as_ref()
            && self.evaluate_requirement_static(filter, tgt, p, Some(card.id))
        {
            mana_cost.reduce_by_cost(less);
        }
        if let Some((_, ref sac_cost)) = offering_pick {
            mana_cost.reduce_by_cost(sac_cost);
        }
        apply_spell_cost_floor(self, &mut mana_cost);
        // CR 601.2g — float-spend confirmation. The spell card is back-in-hand
        // safe to restore (nothing else is committed yet — pitch/gy-exile/
        // return happen after payment), so suspend and replay the alt cast.
        // CR 601.2g float-spend confirmation is a *mana payment* question,
        // so it keys on `manual_mana` — the flag that exists for exactly this
        // rule — rather than `wants_ui`, which bot seats also set. Prompting
        // a bot here is the same livelock as the {X} and additional-cost
        // modals: the suspend returns `Ok`, so the probe reports the cast as
        // legal, and the failed replay is rolled back with the decision
        // restored. Latent rather than observed only because the bot stopped
        // floating mana when it stopped pre-tapping its board.
        if spend_float.is_none()
            && self.players[p].manual_mana
            && !mana_cost.symbols.is_empty()
            && self.float_spend_is_optional(p, &mana_cost, &card.definition.spell_kind())
        {
            let float_summary = self.protectable_float(p, &mana_cost).summary();
            let name = card.definition.name;
            self.players[p].hand.push(card);
            self.pending_decision = Some(crate::game::types::PendingDecision {
                decision: crate::decision::Decision::OptionalTrigger {
                    source: card_id,
                    description: format!(
                        "Spend leftover floating mana ({float_summary}) to cast {name}? (No keeps it and taps lands)"
                    ),
                },
                resume: crate::game::types::ResumeContext::ActionFloatConfirm {
                    actor: p,
                    action: Box::new(GameAction::CastSpellAlternative {
                        card_id,
                        pitch_card,
                        target,
                        additional_targets,
                        mode,
                        x_value,
                    }),
                },
            });
            return Ok(vec![]);
        }
        let forced_only = self.players[p].manual_mana;
        let alt_snapshot = self.snapshot_payment_state(p);
        let receipt = match self.try_pay_after_snapshot_mode(
            p, &mana_cost, alt_snapshot, forced_only, &card.definition.spell_kind(), spend_float,
        ) {
            Ok(r) => r,
            Err(e) => {
                self.players[p].hand.push(card);
                return Err(e);
            }
        };
        self.note_cast_payment_riders(&receipt, &card.definition.spell_kind());
        self.pay_life_cost(p, receipt.side_effects.life_lost);
        let alt_mana_spent = receipt
            .pool_before
            .total()
            .saturating_sub(self.players[p].mana_pool.total());
        let mut auto_events = receipt.auto_events;

        // Pay the life portion of the alt cost (CR 119.4; applied
        // amount honors cannot-lose replacements).
        if alt.life_cost > 0 {
            let applied = self.adjust_life_applied(p, -(alt.life_cost as i32));
            if applied < 0 {
                auto_events.push(GameEvent::LifeLost {
                    player: p,
                    amount: (-applied) as u32,
                });
            }
        }

        // Discard the alt-cost picks (CR 601.2b), firing the normal discard
        // events so "whenever you discard" payoffs see them.
        for did in std::mem::take(&mut discard_picks) {
            self.discard_card(p, did, &mut auto_events);
        }

        // Exile the pitch card from hand if required.
        if alt.exile_filter.is_some()
            && let Some(pitch_id) = pitch_card
            && let Some(pitch) = self.players[p].remove_from_hand(pitch_id)
        {
            let cid = pitch.id;
            self.exile.push(pitch);
            auto_events.push(GameEvent::PermanentExiled { card_id: cid });
        }

        // Exile-N-from-graveyard additional cost. Validated up front;
        // commit the moves now that mana and life are paid. Emits
        // `CardLeftGraveyard` per exile so payoffs that count cards
        // leaving the graveyard (Ark of Hunger, Wilt in the Heat) see
        // the event stream.
        for gy_cid in &exile_gy_picks {
            if let Some(exiled) = Self::take_card(&mut self.players[p].graveyard, *gy_cid) {
                self.exile.push(exiled);
                self.players[p].cards_exiled_this_turn =
                    self.players[p].cards_exiled_this_turn.saturating_add(1);
                auto_events.push(GameEvent::CardLeftGraveyard {
                    player: p,
                    card_id: *gy_cid,
                });
                self.players[p].cards_left_graveyard_this_turn =
                    self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
            }
        }

        // CR 702.119 — Emerge: sacrifice the emerge creature now that the
        // (reduced) mana cost is paid.
        if let Some(sac_cid) = emerge_sac
            && self.battlefield_find(sac_cid).is_some()
        {
            auto_events.push(GameEvent::PermanentSacrificed { card_id: sac_cid, who: p });
            let mut die_evs = self.remove_to_graveyard_with_triggers(sac_cid);
            auto_events.append(&mut die_evs);
        }

        // CR 702.48 — Offering: sacrifice the offered creature now that the
        // (reduced) cost is paid.
        if let Some((sac_cid, _)) = &offering_pick
            && self.battlefield_find(*sac_cid).is_some()
        {
            auto_events.push(GameEvent::PermanentSacrificed { card_id: *sac_cid, who: p });
            let mut die_evs = self.remove_to_graveyard_with_triggers(*sac_cid);
            auto_events.append(&mut die_evs);
        }

        // Tap additional cost: tap the picked creatures now that mana/life
        // are paid.
        for tap_cid in &tap_picks {
            if let Some(c) = self.battlefield_find_mut(*tap_cid) {
                c.tapped = true;
                auto_events.push(GameEvent::PermanentTapped {
                    card_id: *tap_cid,
                    actor: Some(p),
                    as_attacker: false,
                });
            }
        }

        // "Have an opponent gain N life" additional cost (Invigorate). The
        // auto-picker feeds the opponent who's furthest behind.
        if alt.opponent_gains_life > 0
            && let Some(opp) = (0..self.players.len())
                .filter(|&o| o != p && !self.players[o].eliminated)
                .min_by_key(|&o| self.players[o].life)
        {
            let applied = self.adjust_life_applied(opp, alt.opponent_gains_life as i32);
            if applied > 0 {
                auto_events
                    .push(GameEvent::LifeGained { player: opp, amount: applied as u32 });
            }
        }

        // Sacrifice additional cost: sacrifice the picked permanents now
        // that mana/life are paid (CR 701.16). Fires dies/sacrifice triggers.
        for sac_cid in &sacrifice_picks {
            if self.battlefield_find(*sac_cid).is_some() {
                auto_events.push(GameEvent::PermanentSacrificed { card_id: *sac_cid, who: p });
                let mut die_evs = self.remove_to_graveyard_with_triggers(*sac_cid);
                auto_events.append(&mut die_evs);
            }
        }

        // Return-to-hand additional cost: bounce the picked permanents to
        // their owners' hands now that mana/life are paid. Reuses the full
        // `move_card_to` battlefield-exit path (combat removal, continuous-
        // effect cleanup, linked-exile return).
        for ret_cid in &return_picks {
            let owner = self.battlefield_find(*ret_cid).map(|c| c.owner);
            if let Some(owner) = owner {
                let ret_ctx = EffectContext::for_spell(owner, None, 0, 0);
                self.move_card_to(
                    *ret_cid,
                    &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::You),
                    &ret_ctx,
                    &mut auto_events,
                );
            }
        }

        // Overload / alt-effect override: swap the spell's resolution
        // effect to the alternative version so it resolves with "each"
        // instead of "target" semantics (or whatever the override says).
        if let Some(override_effect) = alt.effect_override {
            std::sync::Arc::make_mut(&mut card.definition).effect = override_effect;
        }

        auto_events.push(GameEvent::SpellCast {
            player: p,
            card_id,
            face: CastFace::Front,
        });
        let events = auto_events;
        self.finalize_cast(
            p,
            card,
            target,
            additional_targets,
            mode,
            x_value.unwrap_or(0),
            0,
            alt_mana_spent,
            true,
        );
        Ok(events)
    }


    /// True if `player` is restricted to sorcery-only spell timing by an
    /// opponent's `StaticEffect::OpponentsSorceryTimingOnly` (Teferi, Time
    /// Raveler's static "Each opponent can cast spells only any time they
    /// could cast a sorcery"). Walked by `cast_spell` ahead of the
    /// is-instant-speed timing check.
    pub(crate) fn player_locked_to_sorcery_timing(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller != player
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::OpponentsSorceryTimingOnly)
                })
        })
    }

    /// CR 702.8 — true if a permanent `player` controls grants `card` flash via
    /// a `ControllerSpellsHaveFlash` (filtered) or `ControllerSorceriesAsFlash`
    /// (Teferi, Time Raveler; Hypersonic Dragon) static. Excludes the
    /// card-intrinsic `SelfFlashIf` path, which each cast site handles inline.
    /// CR 601.2b — "you may cast this spell as though it had flash if you pay
    /// [cost] more". Returns the surcharge only when the caster is actually
    /// outside sorcery timing, so a main-phase cast pays the printed cost.
    pub fn flash_surcharge_for<'a>(
        &self,
        player: usize,
        card: &'a CardInstance,
    ) -> Option<&'a crate::mana::ManaCost> {
        let extra = card.definition.flash_surcharge.as_ref()?;
        (!self.can_cast_sorcery_speed(player) && !self.player_locked_to_sorcery_timing(player))
            .then_some(extra)
    }

    pub(crate) fn battlefield_grants_flash(&self, player: usize, card: &CardInstance) -> bool {
        use crate::effect::StaticEffect;
        if self.player_locked_to_sorcery_timing(player) {
            return false;
        }
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| match &sa.effect {
                StaticEffect::AnyPlayerSpellsHaveFlash { filter } => {
                    self.evaluate_requirement_on_card(filter, card, player)
                }
                _ if c.controller != player => false,
                StaticEffect::ControllerSpellsHaveFlash { filter } => {
                    self.evaluate_requirement_on_card(filter, card, player)
                }
                StaticEffect::ControllerSorceriesAsFlash => card.definition.is_sorcery(),
                _ => false,
            })
        })
    }

    /// Validate that a target is legally targetable by the given controller.
    ///
    /// Returns an error if the target has Hexproof (opponent) or Shroud (anyone),
    /// or has Protection from the caster's color identity. For player targets,
    /// also checks the `ControllerHasHexproof` static (Leyline of Sanctity).
    ///
    /// **Ward** — if the target permanent has `Keyword::Ward(n)` and the caster
    /// is an opponent, the caster must have `{n}` generic mana available.
    /// This check is read-only; use `pay_ward_cost` after a successful check
    /// to actually deduct the mana.
    pub fn check_target_legality(&self, target: &Target, caster: usize) -> Result<(), GameError> {
        self.check_target_legality_with_source(target, caster, None)
    }

    /// Same as [`check_target_legality`] but also enforces CR 115.5 — "a
    /// spell or ability on the stack is an illegal target for itself" —
    /// when `source_card_id` is provided. Used by the cast pipeline for
    /// spells like Stifle/Squelch that target stack spells/abilities;
    /// passing the casting spell's own `CardId` rejects a self-target
    /// at cast time.
    pub(crate) fn check_target_legality_with_source(
        &self,
        target: &Target,
        caster: usize,
        source_card_id: Option<CardId>,
    ) -> Result<(), GameError> {
        // CR 801.4 — objects and players outside the caster's range of
        // influence can't be targeted by their spells or abilities.
        let out_of_range = match target {
            Target::Player(p) => !self.player_in_range_of(caster, *p),
            Target::Permanent(c) => !self.object_in_range_of(caster, *c),
        };
        if out_of_range {
            return Err(GameError::InvalidTarget);
        }
        // CR 702.26b — a phased-out permanent is treated as though it doesn't
        // exist, so nothing can target it.
        if let Target::Permanent(c) = target
            && self.phased_out.iter().any(|p| p.id == *c)
        {
            return Err(GameError::InvalidTarget);
        }
        let cid = match target {
            Target::Player(p) => {
                if self.player_has_static_shroud(*p)
                    || (*p != caster
                        && self.player_has_static_hexproof(*p)
                        && !self.player_ignores_hexproof(caster))
                {
                    return Err(GameError::TargetHasHexproof(crate::card::CardId(0)));
                }
                // Protection from everything (The One Ring) — can't be
                // targeted by any spell or ability.
                if self.players[*p].protected_from_everything {
                    return Err(GameError::InvalidTarget);
                }
                return Ok(());
            }
            Target::Permanent(c) => c,
        };
        // CR 115.5: A spell or ability on the stack is an illegal target
        // for itself. When the cast pipeline passes its own source id,
        // reject a target matching that id. Catches Spell Burst /
        // hypothetical "counter target spell" trying to point at itself
        // mid-cast.
        if let Some(src) = source_card_id
            && *cid == src
        {
            return Err(GameError::InvalidTarget);
        }
        let Some(card) = self.battlefield_find(*cid) else {
            // Underworld Cerberus — cards in graveyards can't be targeted.
            if self.graveyard_cards_untargetable()
                && self.players.iter().any(|pl| pl.graveyard.iter().any(|c| c.id == *cid))
            {
                return Err(GameError::InvalidTarget);
            }
            return Ok(());
        };
        // Read layer-computed keywords (CR 613) so granted *and* stripped
        // Hexproof/Shroud are honored — e.g. Nowhere to Run removing an
        // opponent's hexproof makes the creature targetable.
        let controller = card.controller;
        // CR 702.18 — Autumn Willow's waiver makes it targetable by one
        // player's spells and abilities as though it had no shroud.
        // Peace Talks — nothing at all can be targeted for its two turns.
        if self.truce_active() {
            return Err(GameError::TargetHasShroud(*cid));
        }
        if self.permanent_has_keyword(*cid, &Keyword::Shroud)
            && !self.shroud_waivers.contains(&(*cid, caster))
        {
            return Err(GameError::TargetHasShroud(*cid));
        }
        if self.permanent_has_keyword(*cid, &Keyword::Hexproof)
            && controller != caster
            && !self.player_ignores_creature_hexproof(caster)
        {
            return Err(GameError::TargetHasHexproof(*cid));
        }
        // Tomik — a player's lands can't be targeted by an opponent's spells
        // or abilities.
        if card.definition.is_land()
            && controller != caster
            && self.player_lands_untargetable_by_opponents(controller)
        {
            return Err(GameError::InvalidTarget);
        }
        // Artifact Ward — "can't be the target of abilities from [filter]
        // sources". Only ability sources reach here with a `source_card_id` on
        // the battlefield; a spell's own cast gate is upstream.
        if let Some(src) = source_card_id
            && self
                .computed_permanent(*cid)
                .into_iter()
                .flat_map(|cp| cp.keywords.to_vec())
                .any(|k| match k {
                    Keyword::CantBeTargetedByAbilitiesFromMatching(f) => self
                        .evaluate_requirement_static(&f, &Target::Permanent(src), controller, Some(src)),
                    _ => false,
                })
        {
            return Err(GameError::InvalidTarget);
        }
        // Ward is enforced via triggered abilities on the stack (CR 702.21a),
        // not as a pre-flight targeting restriction. The caster CAN target a
        // Ward creature — the Ward trigger fires and counters the spell unless
        // the caster pays the Ward cost at resolution time.
        Ok(())
    }

    /// True while any player controls a `LandsCantEnterTheBattlefield` source
    /// (Worms of the Earth).
    pub(crate) fn lands_cant_enter_the_battlefield(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::LandsCantEnterTheBattlefield))
        })
    }

    /// True while any player controls a `GraveyardCardsUntargetable` source
    /// (Underworld Cerberus).
    pub(crate) fn graveyard_cards_untargetable(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::GraveyardCardsUntargetable))
        })
    }

    /// True while `player` controls a `LandsUntargetableByOpponents` source
    /// (Tomik, Distinguished Advokist).
    pub(crate) fn player_lands_untargetable_by_opponents(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::LandsUntargetableByOpponents)
                })
        })
    }

    /// CR 601.2c — Flagbearer. Every battlefield Flagbearer `actor` could
    /// legally target, when an opponent of `actor` has the restriction up.
    /// Empty when the restriction is inactive, so callers can treat "no
    /// candidates" as "no restriction".
    pub(crate) fn flagbearer_candidates(&self, actor: usize) -> Vec<CardId> {
        use crate::effect::StaticEffect;
        let restricted = self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::FlagbearersMustBeTargeted))
                && !self.same_team(c.controller, actor)
        });
        if !restricted {
            return Vec::new();
        }
        self.battlefield
            .iter()
            .filter(|c| {
                self.computed_permanent(c.id)
                    .is_some_and(|cp| cp.subtypes.creature_types.contains(&crate::card::CreatureType::Flagbearer))
                    && self.check_target_legality(&Target::Permanent(c.id), actor).is_ok()
            })
            .map(|c| c.id)
            .collect()
    }

    /// CR 601.2c — true when `chosen` skips an available Flagbearer that one of
    /// `slot_filters` would have accepted ("must choose at least one … if able").
    pub(crate) fn flagbearer_violation(
        &self,
        actor: usize,
        chosen: &[Target],
        slot_filters: &[Option<crate::card::SelectionRequirement>],
    ) -> bool {
        let candidates = self.flagbearer_candidates(actor);
        if candidates.is_empty() {
            return false;
        }
        if chosen.iter().any(|t| matches!(t, Target::Permanent(c) if candidates.contains(c))) {
            return false;
        }
        // "If able": only a violation when some declared slot would have
        // accepted a Flagbearer.
        slot_filters.iter().any(|f| {
            candidates.iter().any(|&fb| match f {
                Some(filter) => {
                    self.evaluate_requirement(filter, &Target::Permanent(fb), actor)
                }
                None => true,
            })
        })
    }

    /// True while `player` controls a `ControllerCantCastPermanentSpells`
    /// source (Codie, Vociferous Codex).
    pub(crate) fn player_cant_cast_permanent_spells(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::ControllerCantCastPermanentSpells)
                })
        })
    }

    /// True while `player` controls a `ControllerCantCastNoncreatureSpells`
    /// static (Nikya of the Old Ways).
    pub(crate) fn player_cant_cast_noncreature_spells(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.seat_static_sources(player).any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect, StaticEffect::ControllerCantCastNoncreatureSpells)
            })
        })
    }

    /// True while `player` controls a `ControllerCantCastInstantsOrSorceries`
    /// static (Hymn of the Wilds, from the command zone).
    pub(crate) fn player_cant_cast_instants_or_sorceries(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.seat_static_sources(player).any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect, StaticEffect::ControllerCantCastInstantsOrSorceries)
            })
        })
    }

    /// CR 506 — Hand to Hand: while any copy is in play and the turn is in a
    /// combat step, instants can't be cast and non-mana abilities can't be
    /// activated.
    pub(crate) fn combat_spell_lock_active(&self) -> bool {
        use crate::effect::StaticEffect;
        self.step.is_combat_phase()
            && self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        self.active_static(&sa.effect, c),
                        Some(StaticEffect::NoInstantsOrAbilitiesDuringCombat)
                    )
                })
            })
    }

    /// True when an opponent of `player` controls an
    /// `OpponentsCantCastMatching` static whose filter matches `card`
    /// (Llawan, Cephalid Empress — "your opponents can't cast blue creature
    /// spells").
    pub(crate) fn opponent_locks_cast_of(
        &self,
        player: usize,
        card: &crate::card::CardInstance,
    ) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(
                    &sa.effect,
                    StaticEffect::OpponentsCantCastMatching { filter }
                        if self.evaluate_requirement_on_card(filter, card, player)
                )
            }) && !self.same_team(c.controller, player)
        })
    }

    /// City in a Bottle — a symmetric `PlayersCantPlayMatching` static in play
    /// whose filter matches `card`. Binds every seat, its controller included.
    pub(crate) fn play_locked_for_all(&self, player: usize, card: &crate::card::CardInstance) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| {
                matches!(
                    &sa.effect,
                    StaticEffect::PlayersCantPlayMatching { filter }
                        if self.evaluate_requirement_on_card(filter, card, player)
                )
            })
        })
    }

    /// True while `player` controls a `ControllerCantCastCreatureSpells`
    /// static (Grid Monitor).
    pub(crate) fn player_cant_cast_creature_spells(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::ControllerCantCastCreatureSpells)
                })
        })
    }

    /// Gaddock Teeg lock — true if `card` is a noncreature spell barred from
    /// being cast by some `NoncreatureSpellsCantBeCastIf` static anywhere on
    /// the battlefield (global, all players).
    pub fn noncreature_spell_cast_locked(&self, card: &crate::card::CardDefinition) -> bool {
        use crate::effect::StaticEffect;
        if card.is_creature() {
            return false;
        }
        let mv = card.cost.cmc();
        let has_x = card.cost.has_x();
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| match sa.effect {
                StaticEffect::NoncreatureSpellsCantBeCastIf { min_mana_value, or_has_x } => {
                    mv >= min_mana_value || (or_has_x && has_x)
                }
                // Sanctum Prelate — locked only at the exact chosen mana value.
                StaticEffect::NoncreatureSpellsWithChosenManaValueCantBeCast => {
                    c.chosen_number == Some(mv)
                }
                _ => false,
            })
        })
    }

    /// CR 702.16c — a permanent with protection from a quality can't be the
    /// target of an *ability* from a source with that quality. `source` is the
    /// ability's source permanent. Mirrors the cast-time spell gate but reads
    /// the source's qualities (color / creature-ness / creature type) rather
    /// than a spell's colors.
    pub fn ability_target_has_protection(&self, target: &Target, source: CardId) -> bool {
        let Target::Permanent(tid) = target else {
            // Player target: only the turn-scoped hexproof-from-color grant
            // (Veil of Summer) applies — and only against opponents' abilities.
            if let Target::Player(tp) = target {
                let src_controller = self.battlefield_find(source).map(|c| c.controller);
                if let Some(srcc) = src_controller
                    && srcc != *tp
                    && !self.players[*tp].hexproof_from_colors_this_turn.is_empty()
                    && let Some(src) = self.computed_permanent(source)
                {
                    return self.players[*tp]
                        .hexproof_from_colors_this_turn
                        .iter()
                        .any(|c| src.colors.contains(c));
                }
            }
            return false;
        };
        let Some(tgt) = self.computed_permanent(*tid) else { return false };
        let tgt_controller = tgt.controller;
        let src_is_opponent = self
            .battlefield_find(source)
            .is_some_and(|c| c.controller != tgt_controller);
        // CR 702.11d — "hexproof from activated and triggered abilities":
        // an opponent's ability simply can't target it (Volatile Stormdrake).
        if src_is_opponent
            && tgt.keywords.iter().any(|kw| matches!(kw, Keyword::HexproofFromAbilities))
        {
            return true;
        }
        let printed_hexproof_color = tgt.keywords.iter().any(|kw| {
            matches!(
                kw,
                Keyword::HexproofFromColor(_)
                    | Keyword::HexproofFromMonocolored
                    | Keyword::HexproofFromMulticolored
            )
        });
        let turn_hexproof_color = !self.players[tgt_controller]
            .hexproof_from_colors_this_turn
            .is_empty();
        if !tgt.keywords.iter().any(|kw| {
            matches!(
                kw,
                Keyword::Protection(_)
                    | Keyword::ProtectionFromCreatures
                    | Keyword::ProtectionFromCreatureType(_)
                    | Keyword::ProtectionFromMatching(_)
                    | Keyword::ProtectionFromManaValueExcept(_)
                    | Keyword::ProtectionFromManaValueParity { .. }
                    | Keyword::ProtectionFromMulticolored
                    | Keyword::ProtectionFromMonocolored
                    | Keyword::ProtectionFromCardType(_)
                    | Keyword::ProtectionFromOwnColors
                    | Keyword::ProtectionFromEverything
                    | Keyword::HexproofExceptColors(_)
            )
        }) && !(src_is_opponent && (printed_hexproof_color || turn_hexproof_color))
        {
            return false;
        }
        let Some(src) = self.computed_permanent(source) else { return false };
        let src_is_creature = src.card_types.contains(&CardType::Creature);
        let src_mv = self
            .battlefield_find(source)
            .map(|c| c.definition.cost.cmc())
            .unwrap_or(0);
        // Hexproof from [color] (printed or Veil's turn grant) blocks an
        // opponent's same-color ability.
        if src_is_opponent {
            let blocks_color = |color: &ManaColor| src.colors.contains(color);
            if turn_hexproof_color
                && self.players[tgt_controller]
                    .hexproof_from_colors_this_turn
                    .iter()
                    .any(blocks_color)
            {
                return true;
            }
            if tgt.keywords.iter().any(|kw| match kw {
                Keyword::HexproofFromColor(c) => src.colors.contains(c),
                // CR 702.11f — an exactly-one-color source.
                Keyword::HexproofFromMonocolored => src.colors.len() == 1,
                Keyword::HexproofFromMulticolored => src.colors.len() >= 2,
                _ => false,
            }) {
                return true;
            }
        }
        tgt.keywords.iter().any(|kw| match kw {
            Keyword::Protection(color) => src.colors.contains(color),
            // CR 702.16 — "protection from its colors" (Earnest Fellowship).
            Keyword::ProtectionFromOwnColors => {
                tgt.colors.iter().any(|c| src.colors.contains(c))
            }
            Keyword::ProtectionFromCreatures => src_is_creature,
            Keyword::ProtectionFromCreatureType(ty) => src.subtypes.creature_types.contains(ty),
            Keyword::ProtectionFromMatching(f) => {
                self.evaluate_requirement_static(f, &Target::Permanent(source), tgt_controller, None)
            }
            Keyword::ProtectionFromManaValueExcept(n) => src_mv != *n,
            Keyword::ProtectionFromManaValueParity { odd } => (src_mv % 2 == 1) == *odd,
            Keyword::ProtectionFromMulticolored => src.colors.len() >= 2,
            Keyword::ProtectionFromMonocolored => src.colors.len() == 1,
            Keyword::ProtectionFromCardType(t) => src.card_types.contains(t),
            Keyword::ProtectionFromEverything => true,
            // "Abilities from nongreen sources opponents control can't target
            // this" (Thrun) — opponent's source sharing none of the colors.
            Keyword::HexproofExceptColors(colors) => {
                src_is_opponent && !colors.iter().any(|c| src.colors.contains(c))
            }
            _ => false,
        })
    }

    /// Card types `player` has protection from via
    /// `StaticEffect::YouAndCreaturesProtectionFromChosenCardType`
    /// permanents they control (Serra's Emissary).
    pub(crate) fn player_protection_card_types(&self, player: usize) -> Vec<crate::card::CardType> {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == player)
            .filter(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::YouAndCreaturesProtectionFromChosenCardType)
                })
            })
            .filter_map(|c| c.chosen_card_type.clone())
            .collect()
    }

    /// True if `player` controls any permanent granting "you have hexproof"
    /// via `StaticEffect::ControllerHasHexproof` (Leyline of Sanctity).
    pub fn player_has_static_hexproof(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.players.get(player).is_some_and(|p| p.hexproof_until_next_turn)
            || self.battlefield.iter().any(|c| {
                c.controller == player
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::ControllerHasHexproof))
            })
    }

    /// Peace Talks — true while its two-turn truce is live: no creature can
    /// attack and nothing can be targeted by a spell or activated ability.
    pub fn truce_active(&self) -> bool {
        self.truce_until_turn.is_some_and(|t| self.turn_number <= t)
    }

    /// CR 702.18 — true if `player` controls a permanent granting "you have
    /// shroud" (Ivory Mask). Unlike hexproof this also blocks the player's own
    /// spells and abilities, and no ignore-hexproof static pierces it.
    pub fn player_has_static_shroud(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        // Peace Talks blankets every player for its two turns.
        if self.truce_active() {
            return true;
        }
        // Gilded Light's turn-scoped grant rides the same check.
        if self.players.get(player).is_some_and(|p| p.shroud_this_turn) {
            return true;
        }
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::ControllerHasShroud))
        })
    }

    /// True if `player` controls a permanent granting "ignore opponents'
    /// creature hexproof" (Glaring Spotlight) — plain `Hexproof` on opponents'
    /// creatures no longer shields them from `player`'s spells and abilities.
    pub(crate) fn player_ignores_creature_hexproof(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.player_ignores_hexproof(player)
            || self.battlefield.iter().any(|c| {
                c.controller == player
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::IgnoreOpponentsCreatureHexproof))
            })
    }

    /// True if an *opponent* of `player` controls an
    /// `OpponentsCantSearchLibraries` static (Ashiok, Dream Render) — `player`
    /// can't search their own library (CR 701.19).
    pub(crate) fn player_search_locked_by_opponent(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::OpponentsCantSearchLibraries))
                && !self.same_team(c.controller, player)
        })
    }

    /// True if `player` controls a permanent granting the broad "ignore
    /// opponents' hexproof" static (Kaya, Bane of the Dead) — plain `Hexproof`
    /// on opponents' permanents *and* opponent players no longer shields them.
    pub(crate) fn player_ignores_hexproof(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::IgnoreOpponentsHexproof))
        })
    }

    /// True when `player` controls a permanent with the "opponents can't make
    /// you sacrifice" static (Sigarda, Host of Herons / Tamiyo). Consulted by
    /// the `Effect::Sacrifice` resolver to skip an opponent-forced sacrifice.
    pub(crate) fn player_cant_be_made_to_sacrifice(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::OpponentsCantMakeYouSacrifice))
        })
    }

    /// The discard sibling of the above (Tamiyo, Collector of Tales).
    /// Consulted by the `Effect::Discard` resolver.
    pub(crate) fn player_cant_be_made_to_discard(&self, player: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::OpponentsCantMakeYouDiscard))
        })
    }

    // Note: `fire_ward_triggers` (the old Ward(u32) version) was removed
    // during the merge — Ward is now enforced via
    // `push_ward_triggers_for_cast` (CR 702.21) which handles the full
    // `WardCost` enum (Mana / Life / Discard / SacrificeCreature).

    /// Push `SpellCast` triggered abilities (e.g. Prowess, Up the Beanstalk)
    /// onto the stack. They will resolve when priority is passed through.
    /// `cast_card` is the id of the spell that just resolved (or just got
    /// cast); the trigger's optional `EventSpec::filter` predicate is
    /// evaluated with `Selector::TriggerSource` bound to this card so
    /// "whenever you cast a spell with property X" filters can read the
    /// cast spell's mana value, color, type, etc.
    pub(crate) fn fire_spell_cast_triggers(
        &mut self,
        controller: usize,
        cast_card: CardId,
        _is_noncreature: bool,
        mana_spent: u32,
        converged_value: u32,
    ) {
        use crate::effect::{EventKind, EventScope};
        // CR 603.7e — one-shot "when you cast your next spell this turn"
        // delayed triggers (Codie). Fire each matching watcher once, with
        // the cast spell bound as the trigger source, and consume it.
        let cast_is_is = self.find_card_anywhere(cast_card).is_some_and(|c| {
            c.definition.card_types.contains(&crate::card::CardType::Instant)
                || c.definition.card_types.contains(&crate::card::CardType::Sorcery)
        });
        let (next_cast, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.delayed_triggers)
            .into_iter()
            .partition(|dt| {
                dt.controller == controller
                    && (matches!(
                        dt.kind,
                        crate::game::types::DelayedKind::YourNextSpellCastThisTurn
                    ) || (cast_is_is
                        && matches!(
                            dt.kind,
                            crate::game::types::DelayedKind::YourNextInstantSorceryCastThisTurn
                        )))
            });
        self.delayed_triggers = rest;
        // Expose the cast spell's mana value so bodies can gate on it
        // (Vivien, Monsters' Advocate — "a creature card with lesser mana
        // value" via `ManaValueLessThanEventAmount`).
        let cast_mv = self
            .find_card_anywhere(cast_card)
            .map(|c| c.definition.cost.cmc())
            .unwrap_or(0);
        for dt in next_cast {
            self.stack.push(
                TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                    .trigger_source(Some(crate::game::effects::EntityRef::Card(cast_card)))
                    .event_amount(cast_mv)
                    .build(),
            );
            // Repeating watchers ("whenever you cast a spell this turn",
            // Rediscover the Way III) survive until cleanup clears them.
            if !dt.fires_once {
                self.delayed_triggers.push(dt);
            }
        }
        // CR 603.7e (name-gated) — "when you cast a spell with the chosen name
        // for the first time this turn" (Medomai's Prophecy III). Only a cast
        // whose name matches the watching source's `named_card` consumes the
        // one-shot; other casts leave it armed.
        let cast_name = self
            .find_card_anywhere(cast_card)
            .map(|c| c.definition.name.to_string());
        let (named_fire, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.delayed_triggers)
            .into_iter()
            .partition(|dt| {
                dt.controller == controller
                    && matches!(dt.kind, crate::game::types::DelayedKind::YourNextNamedSpellThisTurn)
                    && self
                        .battlefield_find(dt.source)
                        .and_then(|s| s.named_card.clone())
                        == cast_name
            });
        self.delayed_triggers = rest;
        for dt in named_fire {
            self.stack.push(
                TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                    .trigger_source(Some(crate::game::effects::EntityRef::Card(cast_card)))
                    .build(),
            );
        }
        // CR 113.10b — permanents under a "loses all abilities" continuous
        // effect (Mercurial Transformation / Turn to Frog) don't fire
        // printed Magecraft / spell-cast triggers. Pre-compute the stripped
        // set so the filter below can drop those listeners.
        let stripped: std::collections::HashSet<CardId> =
            self.permanents_with_abilities_removed().into_iter().collect();
        // Whether the cast spell came from its caster's hand, read off the
        // stack item. Stamped into the trigger context so
        // `Predicate::CastFromHand` (Quandrix, the Proof) reflects the
        // actual cast: cascade / flashback / exile casts read `false`,
        // which stops "spells you cast from your hand have cascade" from
        // re-triggering on the spells it cascades into.
        let cast_from_hand = self
            .stack
            .iter()
            .find_map(|item| match item {
                crate::game::types::StackItem::Spell { card, .. } if card.id == cast_card => {
                    Some(card.cast_from_hand)
                }
                _ => None,
            })
            .unwrap_or(true);
        // Walk every permanent on the battlefield — `YourControl` triggers
        // fire from the caster's permanents, while `OpponentControl` triggers
        // fire from non-caster permanents (Wandering Archaic etc.). The
        // ability's effective controller is its own permanent's controller,
        // *not* the spell-caster's index.
        // A SpellCast trigger applies iff its scope matches the caster.
        let scope_matches = |scope: EventScope, c_controller: usize| match scope {
            EventScope::YourControl => c_controller == controller,
            EventScope::OpponentControl => c_controller != controller,
            EventScope::AnyPlayer => true,
            _ => false,
        };
        // Walk printed triggers AND statics-/equip-granted ones (Red Mage's
        // Rapier's "equipped creature has 'whenever you cast a noncreature
        // spell, …'", Sliver-granted Magecraft): granted abilities fire as
        // though printed on the host. Granted triggers use a sentinel index and
        // are never once-per-turn.
        #[allow(clippy::type_complexity)]
        let mut candidates: Vec<(CardId, usize, Effect, Option<crate::effect::Predicate>, usize, bool)> =
            Vec::new();
        let live: Vec<(CardId, usize)> = self
            .battlefield
            .iter()
            .filter(|c| !stripped.contains(&c.id))
            .map(|c| (c.id, c.controller))
            .collect();
        // Both grant lists are board-level: the per-card shims rebuild them,
        // so asking them per live permanent is O(cards²).
        let trigger_grants = self.trigger_grant_sources();
        let equip_grants = self.equip_granted_trigger_sources();
        for (cid, c_controller) in live {
            let Some(c) = self.battlefield.iter().find(|c| c.id == cid) else { continue };
            for (idx, t) in c.definition.triggered_abilities.iter().enumerate() {
                if t.event.kind == EventKind::SpellCast && scope_matches(t.event.scope, c_controller) {
                    candidates.push((cid, c_controller, t.effect.clone(), t.event.filter.clone(), idx, t.event.once_per_turn));
                }
            }
            for t in self
                .statics_granted_triggers_with(c, &trigger_grants)
                .into_iter()
                .chain(self.equip_granted_triggers_with(c, &equip_grants))
            {
                if t.event.kind == EventKind::SpellCast && scope_matches(t.event.scope, c_controller) {
                    candidates.push((cid, c_controller, t.effect, t.event.filter, usize::MAX, false));
                }
            }
        }
        // CR 902.5 — a Vanguard avatar's cast trigger fires from the command
        // zone (Serra Angel Avatar's "whenever you cast a spell, gain 2 life").
        for (seat, pl) in self.players.iter().enumerate() {
            for c in pl.command.iter().filter(|c| c.command_zone_abilities_active()) {
                for t in &c.definition.triggered_abilities {
                    if t.event.kind == EventKind::SpellCast && scope_matches(t.event.scope, seat) {
                        candidates.push((
                            c.id,
                            seat,
                            t.effect.clone(),
                            t.event.filter.clone(),
                            usize::MAX,
                            false,
                        ));
                    }
                }
            }
        }
        // CR — "Whenever you cast a multicolored spell, you may return this
        // from your graveyard to your hand" (the Dissension Eidolon cycle):
        // a `FromYourGraveyard`-scoped SpellCast trigger fires from its owner's
        // graveyard when that owner is the caster.
        let gy_casters: Vec<(CardId, usize)> = self
            .players
            .iter()
            .enumerate()
            .flat_map(|(owner, pl)| pl.graveyard.iter().map(move |c| (c.id, owner)))
            .filter(|&(_, owner)| owner == controller)
            .collect();
        for (cid, owner) in gy_casters {
            let Some(c) = self.players[owner].graveyard.iter().find(|c| c.id == cid) else {
                continue;
            };
            for t in &c.definition.triggered_abilities {
                if t.event.kind == EventKind::SpellCast
                    && matches!(t.event.scope, EventScope::FromYourGraveyard)
                {
                    candidates.push((cid, owner, t.effect.clone(), t.event.filter.clone(), usize::MAX, false));
                }
            }
        }

        for (source, listener_controller, effect, filter, trig_idx, once_per_turn) in candidates {
            // CR 603.3d — "This ability triggers only once each turn"
            // (Whispering Wizard, Welcoming Vampire-style SpellCast payoffs).
            let once_key = (source, trig_idx);
            if once_per_turn && self.triggered_once_per_turn_used.contains(&once_key) {
                continue;
            }
            if let Some(filter) = filter {
                let ctx = crate::game::effects::EffectContext {
                    controller: listener_controller,
                    source: Some(source),
                    targets: vec![],
                    trigger_source: Some(crate::game::effects::EntityRef::Card(cast_card)),
                    mode: 0,
                    x_value: 0,
                    converged_value,
                    mana_spent,
                    mana_spent_by_color: Vec::new(),
                    source_name: None,
                    cast_from_hand,
                    event_amount: 0,
                    kicked: false,
                    kicked_options: Vec::new(),
                    kick_count: 0,
                    bargained: false,
                    cast_via_mayhem: false,
                    cast_via_waterbend: false,
                    cast_collected_evidence: false,
                    entwined: false,
                    spree_modes: Vec::new(),
                };
                if !self.evaluate_predicate(&filter, &ctx) {
                    continue;
                }
            }
            // The trigger is firing this cast — consume its once-per-turn slot.
            if once_per_turn {
                self.triggered_once_per_turn_used.insert(once_key);
            }
            let auto_target = self.auto_target_for_effect_avoiding(
                &effect,
                listener_controller,
                Some(source),
            );
            // CR 700.2b — pick the mode at push time if the trigger is modal.
            // Powers Prismari Apprentice's modal Magecraft (Scry 1 / +1/+0 EOT):
            // AutoDecider picks mode 0 (Scry); ScriptedDecider::new([Mode(1)])
            // exercises the pump branch.
            let mode = self.pick_trigger_mode(&effect, source, listener_controller);
            // The cast spell's mana value, so "where X is that spell's mana
            // value" riders scale (Shark Typhoon).
            let spell_mv = self
                .stack
                .iter()
                .find_map(|si| match si {
                    StackItem::Spell { card, .. } if card.id == cast_card => {
                        Some(card.definition.cost.cmc())
                    }
                    _ => None,
                })
                .unwrap_or(0);
            // CR 603.x — Harmonic Prodigy / Veyran / Katara: a SpellCast
            // (Magecraft) trigger of a matching-subtype permanent fires an
            // additional time per doubler the controller controls.
            let fires = 1 + ally_trigger_extra_fires(self, listener_controller, source);
            for _ in 0..fires {
                self.stack.push(
                    TriggerPush::new(source, listener_controller, effect.clone())
                        .target(auto_target.clone())
                        .mode(mode)
                        // The cast spell's converge count, so per-cast
                        // `Value::ConvergedValue` reads the iterated spell
                        // (Magmablood / Wildgrowth Archaic).
                        .converged_value(converged_value)
                        // Preserve the cast spell's id for Effect::CopySpell /
                        // Selector::CastSpellTarget.
                        .trigger_source(Some(crate::game::effects::EntityRef::Card(cast_card)))
                        .mana_spent(mana_spent)
                        .event_amount(spell_mv)
                        .build(),
                );
            }
        }
    }

    // ── Payment snapshot / restore ───────────────────────────────────────────

    /// Capture mana pool + tapped state of every permanent owned by `payer`.
    /// Used by the cast/activate/counter paths so a payment that fails
    /// mid-way (after auto-tap has already tapped lands) can be reverted to
    /// pristine state.
    pub(crate) fn snapshot_payment_state(&self, payer: usize) -> PaymentSnapshot {
        PaymentSnapshot {
            pool: self.players[payer].mana_pool.clone(),
            tapped: self
                .battlefield
                .iter()
                .filter(|c| c.owner == payer)
                .map(|c| (c.id, c.tapped))
                .collect(),
        }
    }

    /// Restore the mana pool and tapped state captured by a prior
    /// `snapshot_payment_state`. Skips cards that have since left the
    /// battlefield (the caller is responsible for any zone-change rollback).
    pub(crate) fn restore_payment_state(&mut self, payer: usize, snapshot: PaymentSnapshot) {
        self.players[payer].mana_pool = snapshot.pool;
        for (id, was_tapped) in snapshot.tapped {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id) {
                c.tapped = was_tapped;
            }
        }
    }

    /// Snapshot, auto-tap, and pay `cost` atomically. Returns the auto-tap
    /// events plus the pre-payment pool snapshot (callers like `cast_spell`
    /// use it to compute converge). On payment failure the snapshot is
    /// restored — pool and tapped flags revert to the pre-call state.
    /// CR 702.8 — may `p` cast `card` at instant speed? A card-intrinsic
    /// `SelfFlashIf`, a battlefield grant (Sigarda's Aid), a flash *surcharge*
    /// (Vedalken Orrery-style taxes), or Winding Canyons' turn-scoped
    /// creature-spell permission. The six cast entry points share this.
    pub(crate) fn flash_granted_for(&self, p: usize, card: &crate::card::CardInstance) -> bool {
        let self_flash = !self.player_locked_to_sorcery_timing(p)
            && card.definition.static_abilities.iter().any(|sa| {
                if let crate::effect::StaticEffect::SelfFlashIf { condition } = &sa.effect {
                    let ctx = crate::game::effects::EffectContext::for_spell(p, None, 0, 0);
                    self.evaluate_predicate(condition, &ctx)
                } else {
                    false
                }
            });
        self_flash
            || self.battlefield_grants_flash(p, card)
            || self.flash_surcharge_for(p, card).is_some()
            || (self.players[p].creature_spells_as_flash_this_turn
                && card.definition.is_creature())
    }

    pub(crate) fn try_pay_with_auto_tap(
        &mut self,
        payer: usize,
        cost: &crate::mana::ManaCost,
    ) -> Result<PaymentReceipt, GameError> {
        self.try_pay_with_auto_tap_mode(payer, cost, false)
    }

    /// `try_pay_with_auto_tap`, but `forced_only` gates manual tapping.
    /// When `forced_only` is true (human-initiated casts/activations), the
    /// engine auto-taps *only* when the payment is forced — see
    /// `try_pay_after_snapshot_mode`.
    pub(crate) fn try_pay_with_auto_tap_mode(
        &mut self,
        payer: usize,
        cost: &crate::mana::ManaCost,
        forced_only: bool,
    ) -> Result<PaymentReceipt, GameError> {
        self.try_pay_with_auto_tap_kind(payer, cost, forced_only, &crate::mana::SpellKind::default())
    }

    /// `try_pay_with_auto_tap_mode` with an explicit `SpellKind`, so a cast
    /// path that isn't the plain `cast_spell` (Omen halves) can still let
    /// spend-restricted mana recognize what it's funding.
    pub(crate) fn try_pay_with_auto_tap_kind(
        &mut self,
        payer: usize,
        cost: &crate::mana::ManaCost,
        forced_only: bool,
        kind: &crate::mana::SpellKind,
    ) -> Result<PaymentReceipt, GameError> {
        let snapshot = self.snapshot_payment_state(payer);
        // These paths don't pose the float-spend confirmation, so pass `None`.
        self.try_pay_after_snapshot_mode(payer, cost, snapshot, forced_only, kind, None)
    }

    /// CR 601.2g — the pre-existing floating mana that paying `cost` would sweep
    /// onto the *generic* portion: the player's pool minus the mana that matches
    /// the cost's colored / `{C}` pips (which they'd spend on those pips anyway).
    /// This "excess" is the only float that's discretionary — what the spend
    /// confirmation is actually about ("keep your leftover {R}, or spend it?").
    pub(crate) fn protectable_float(
        &self,
        payer: usize,
        cost: &crate::mana::ManaCost,
    ) -> crate::mana::ManaPool {
        use crate::mana::{Color, ManaPool, ManaSymbol};
        let pool = &self.players[payer].mana_pool;
        let mut colored_need: std::collections::HashMap<Color, u32> = std::collections::HashMap::new();
        let mut colorless_need = 0u32;
        for s in &cost.symbols {
            match s {
                ManaSymbol::Colored(c) => *colored_need.entry(*c).or_default() += 1,
                ManaSymbol::Colorless(n) => colorless_need += *n,
                _ => {}
            }
        }
        let mut protected = ManaPool::default();
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            let excess = pool.amount(c).saturating_sub(*colored_need.get(&c).unwrap_or(&0));
            if excess > 0 {
                protected.add(c, excess);
            }
        }
        let cl_excess = pool.colorless_amount().saturating_sub(colorless_need);
        if cl_excess > 0 {
            protected.add_colorless(cl_excess);
        }
        protected
    }

    /// CR 601.2g — should paying `cost` for `payer` prompt "spend your leftover
    /// floating mana, or tap lands?" True iff there's *excess* float (off the
    /// cost's colored pips), the cost has a generic portion that would consume
    /// it, AND the cost is still payable while keeping that excess (so spending
    /// it was avoidable, not the only legal source). Pip-matching float is never
    /// in question — it's spent on its pip regardless.
    pub(crate) fn float_spend_is_optional(
        &self,
        payer: usize,
        cost: &crate::mana::ManaCost,
        kind: &crate::mana::SpellKind,
    ) -> bool {
        use crate::mana::ManaSymbol;
        // Prefilled pool: if the floating mana already covers the *whole* cost,
        // the player arranged it deliberately (e.g. via the manual-tap flow) —
        // pay from it directly, no prompt. This is distinct from the partial
        // case below (Aberrant Manawurm {3}{G} with {R}{G} floating doesn't
        // cover the cost, so it still prompts about the excess).
        if self.players[payer].mana_pool.clone().pay_for_spell(cost, kind).is_ok() {
            return false;
        }
        let protected = self.protectable_float(payer, cost);
        if protected.total() == 0 {
            return false; // no discretionary float — only pip-matching mana
        }
        // Excess is only swept up by a generic ({N}) portion; with none, it just
        // stays floating and there's nothing to ask about.
        let generic_need: u32 = cost
            .symbols
            .iter()
            .map(|s| if let ManaSymbol::Generic(n) = s { *n } else { 0 })
            .sum();
        if generic_need == 0 {
            return false;
        }
        // Can the cost be paid while *keeping* the excess (remove it, pay from
        // the remaining pip-matching float + freshly-tapped sources)? If so, the
        // excess wasn't forced — offer the choice. Dry-run on a clone.
        let mut probe = self.clone();
        probe.players[payer].mana_pool.remove_pool(&protected);
        let snap = probe.snapshot_payment_state(payer);
        probe
            .try_pay_after_snapshot_mode(payer, cost, snap, false, kind, None)
            .is_ok()
    }

    /// Pay `cost` for `payer`, auto-tapping mana sources as needed.
    /// Used with a snapshot the caller already captured — for paths that
    /// mutate state between snapshot and payment (convoke taps creatures
    /// in between, `activate_ability` applies its tap-cost in between).
    ///
    /// `forced_only` implements "proper tapping" for human players (CR
    /// 601.2g — the active player chooses which mana sources to tap):
    /// - If the pool already covers the cost, pay from it directly (the
    ///   player has arranged their mana).
    /// - Otherwise the engine auto-taps **only if the payment is forced** —
    ///   i.e. after a full auto-tap, no untapped source the player controls
    ///   *could have contributed to this cost* remains. If a relevant
    ///   untapped source is left over, the player had a real choice, so the
    ///   cast is rejected with `ManualTapRequired` (rolled back) and they
    ///   tap manually before re-submitting.
    ///
    /// `forced_only` is false for bots, scripted tests, and engine-driven
    /// auto-pays (Counter-unless-paid, "pay X or sacrifice"), which keep
    /// the original full auto-tap behavior.
    pub(crate) fn try_pay_after_snapshot_mode(
        &mut self,
        payer: usize,
        cost: &crate::mana::ManaCost,
        snapshot: PaymentSnapshot,
        forced_only: bool,
        kind: &crate::mana::SpellKind,
        // CR 601.2g float-spend choice: `Some(false)` = the player declined to
        // spend their pre-existing floating mana, so pay from freshly-tapped
        // sources and keep the float; `Some(true)`/`None` = normal payment
        // (spend float if it covers the cost). Only meaningful on the
        // `forced_only` (human) path.
        spend_float: Option<bool>,
    ) -> Result<PaymentReceipt, GameError> {
        // CR 609.4b — Mycosynth Lattice's "players may spend mana as though it
        // were mana of any color": relax the coloured pips before anything
        // downstream (auto-tap, float protection, `pay_for_spell`) reads them.
        let relaxed = self.relax_cost_colors_for_spell(Some(payer), cost, kind);
        let cost: &crate::mana::ManaCost = &relaxed;
        if forced_only {
            // "Keep my leftover floating mana": lift out only the *excess*
            // float (the off-pip mana that would hit the generic), pay the cost
            // from the remaining pip-matching float + freshly-tapped sources,
            // then restore the excess. Pip-matching float is still spent on its
            // pip (auto-tap accounts for the pool). We only reach here after the
            // cast confirmed this is payable (see `float_spend_is_optional`); if
            // it somehow can't, restore the excess and fall through to spending
            // it (it was forced after all).
            if spend_float == Some(false) {
                let protected = self.protectable_float(payer, cost);
                if protected.total() > 0 {
                    self.players[payer].mana_pool.remove_pool(&protected);
                    let src_snapshot = self.snapshot_payment_state(payer);
                    match self.try_pay_after_snapshot_mode(payer, cost, src_snapshot, false, kind, None) {
                        Ok(mut receipt) => {
                            self.players[payer].mana_pool.absorb(&protected);
                            receipt.pool_before.absorb(&protected);
                            return Ok(receipt);
                        }
                        Err(_) => {
                            self.players[payer].mana_pool.absorb(&protected);
                            // fall through to the normal (float-spending) path
                        }
                    }
                }
            }
            // Fast path: the player already has the mana floating — pay it.
            if self.players[payer].mana_pool.clone().pay_for_spell(cost, kind).is_ok() {
                let pool_before = self.players[payer].mana_pool.clone();
                let side_effects = self.players[payer]
                    .mana_pool
                    .pay_for_spell(cost, kind)
                    .expect("pool covered the cost a line ago");
                return Ok(PaymentReceipt { auto_events: vec![], side_effects, pool_before });
            }
            // A from-hand mana source (Elvish Spirit Guide) that could chip
            // in is a payment option the auto-tapper can't exercise — stop
            // before tapping anything so the player decides.
            if self.hand_mana_source_could_pay(payer, cost) {
                return Err(GameError::ManualTapRequired { cost: cost.summary() });
            }
            // Eagerly tap for the *forced colored* pips — their colour can
            // only come from those sources, so there's no choice to leave
            // the player (which Forest pays {G} doesn't matter). This mana
            // stays floating in the pool; if the rest of the cost needs a
            // manual tap, the player only has to tap the ambiguous part.
            // A colour producible by sources of different kind/signature
            // (Forest vs Mox Emerald) is NOT forced — leave those pips out
            // of the eager tap so the choice check below can prompt.
            let color_is_forced = |col: ManaColor| -> bool {
                use std::collections::HashSet;
                let need = cost
                    .symbols
                    .iter()
                    .filter(|s| matches!(s, crate::mana::ManaSymbol::Colored(c) if *c == col))
                    .count() as u32;
                if self.players[payer].mana_pool.amount(col) >= need {
                    return true; // pool covers the pip — no tap involved
                }
                let sigs: HashSet<(u8, Vec<ManaColor>)> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == payer
                            && !c.tapped
                            && c.definition.activated_abilities.iter().any(|a| {
                                is_mana_ability(&a.effect)
                                    && effect_produces_color(&a.effect, col)
                            })
                    })
                    .map(source_kind_signature)
                    .collect();
                sigs.len() <= 1
            };
            let colored_only = crate::mana::ManaCost {
                symbols: cost
                    .symbols
                    .iter()
                    .filter(|s| matches!(s, crate::mana::ManaSymbol::Colored(c) if color_is_forced(*c)))
                    .copied()
                    .collect(),
            };
            let mut events = if colored_only.symbols.is_empty() {
                Vec::new()
            } else {
                self.auto_tap_for_cost(payer, &colored_only)
            };
            // With the colored pips covered, does the remaining (generic)
            // part still involve a genuine choice of which source to tap?
            if self.payment_requires_manual_choice(payer, cost) {
                // Leave the forced colored mana floating; the player taps
                // the ambiguous remainder, then the cast completes.
                return Err(GameError::ManualTapRequired { cost: cost.summary() });
            }
            // No remaining choice — finish auto-tapping and pay.
            let mut more = self.auto_tap_for_cost(payer, cost);
            events.append(&mut more);
            let pool_after_auto_tap = self.players[payer].mana_pool.clone();
            return match self.players[payer].mana_pool.pay_for_spell(cost, kind) {
                Ok(side_effects) => {
                    Ok(PaymentReceipt { auto_events: events, side_effects, pool_before: pool_after_auto_tap })
                }
                Err(e) => {
                    self.restore_payment_state(payer, snapshot);
                    Err(GameError::Mana(e))
                }
            };
        }

        let auto_events = self.auto_tap_for_cost_filtered(payer, cost, kind.creature_mana_only);
        // Snapshot the pool *after* auto-tap so `pool_before` reflects the
        // mana actually available to `pay()`. Without this, a player who
        // starts with an empty pool and auto-taps lands to cover the cost
        // shows mana_spent = 0 (pre-auto-tap 0 → post-pay 0), which silently
        // breaks Increment / Opus / converge payoffs that read the
        // difference. The original snapshot is still used for rollback.
        let pool_after_auto_tap = self.players[payer].mana_pool.clone();
        match self.players[payer].mana_pool.pay_for_spell(cost, kind) {
            Ok(side_effects) => Ok(PaymentReceipt {
                auto_events,
                side_effects,
                pool_before: pool_after_auto_tap,
            }),
            Err(e) => {
                // Channel — convert life 1:1 into the colorless shortfall
                // ("you may pay 1 life: add {C}", active until end of turn).
                if self.players[payer].channel_life_for_mana {
                    let life = self.players[payer].life.max(0) as u32;
                    for n in 1..=life {
                        let mut probe = self.players[payer].mana_pool.clone();
                        probe.add_colorless(n);
                        if probe.pay_for_spell(cost, kind).is_err() {
                            continue;
                        }
                        self.pay_life_cost(payer, n);
                        self.players[payer].mana_pool.add_colorless(n);
                        let pool_before = self.players[payer].mana_pool.clone();
                        let side_effects = self.players[payer]
                            .mana_pool
                            .pay_for_spell(cost, kind)
                            .expect("probe covered the cost a line ago");
                        return Ok(PaymentReceipt { auto_events, side_effects, pool_before });
                    }
                }
                self.restore_payment_state(payer, snapshot);
                Err(GameError::Mana(e))
            }
        }
    }

    /// True if `player` controls an untapped mana source that *could have
    /// contributed* to `cost` — used by the forced-only payment path to
    /// detect that a manual tapping choice existed. A source is relevant
    /// when the cost has a generic (or monocolored-hybrid) pip that any
    /// mana satisfies, or when the source can produce a color the cost
    /// requires.
    fn untapped_relevant_source_exists(&self, player: usize, cost: &crate::mana::ManaCost) -> bool {
        self.with_frozen_layers(|g| g.untapped_relevant_source_exists_inner(player, cost))
    }

    fn untapped_relevant_source_exists_inner(
        &self,
        player: usize,
        cost: &crate::mana::ManaCost,
    ) -> bool {
        use crate::mana::ManaSymbol;
        let flexible = cost.symbols.iter().any(|s| {
            matches!(s, ManaSymbol::Generic(n) if *n > 0) || matches!(s, ManaSymbol::MonoHybrid(_, _))
        });
        let cost_colors = cost.colors();
        let scan = self.grant_scan();
        self.battlefield.iter().any(|c| {
            if c.controller != player || c.tapped {
                return false;
            }
            let mana_abilities = self.effective_mana_abilities_with(c.id, &scan);
            if mana_abilities.is_empty() {
                return false;
            }
            // Any mana pays a generic / mono-hybrid-generic pip.
            if flexible {
                return true;
            }
            // Otherwise the source must make a color the cost needs.
            cost_colors
                .iter()
                .any(|col| mana_abilities.iter().any(|(_, a)| effect_produces_color(&a.effect, *col)))
        })
    }

    /// Does paying `cost` for `player` involve a *genuine* choice of which
    /// mana sources to tap — one the engine shouldn't make for them?
    ///
    /// Returns true only when the cost is affordable from untapped sources
    /// AND the player could tap different sources leaving different mana
    /// behind. Forced colored pips (a colour only one kind of source can
    /// make) and interchangeable sources (two of the same basic) are *not*
    /// choices, so they auto-tap. Hybrid / mono-hybrid costs fall back to
    /// the conservative "any relevant untapped source remains" check.
    ///
    /// Only consulted on the forced-only (human) path when the pool doesn't
    /// already cover the cost; `pay()` is still the source of truth for
    /// whether the resulting tap actually pays.
    fn payment_requires_manual_choice(&self, player: usize, cost: &crate::mana::ManaCost) -> bool {
        use crate::mana::{Color, ManaSymbol};
        use std::collections::{HashMap, HashSet};
        // A from-hand mana source (Elvish Spirit Guide) is invisible to the
        // auto-tapper but a real payment option — its presence alone makes
        // the payment a manual choice.
        if self.hand_mana_source_could_pay(player, cost) {
            return true;
        }
        // Hybrids keep the simpler conservative behaviour.
        if cost
            .symbols
            .iter()
            .any(|s| matches!(s, ManaSymbol::Hybrid(_, _) | ManaSymbol::MonoHybrid(_, _)))
        {
            return self.untapped_relevant_source_exists(player, cost);
        }

        // Colored requirement per colour + generic (folding {C} in as
        // generic — payable from any source for the choice analysis).
        let mut need: HashMap<Color, u32> = HashMap::new();
        let mut generic = 0u32;
        for s in &cost.symbols {
            match s {
                ManaSymbol::Colored(c) => *need.entry(*c).or_default() += 1,
                ManaSymbol::Generic(n) | ManaSymbol::Colorless(n) => generic += *n,
                _ => {}
            }
        }

        // Pay colored pips from the pool first; what's left needs sources.
        let pool = &self.players[player].mana_pool;
        let mut pool_used = 0u32;
        let mut colored_from_sources: Vec<(Color, u32)> = Vec::new();
        for (c, k) in &need {
            let from_pool = (*k).min(pool.amount(*c));
            pool_used += from_pool;
            if *k - from_pool > 0 {
                colored_from_sources.push((*c, *k - from_pool));
            }
        }

        // Untapped mana sources, each as its kind + colour-production
        // signature (a Forest and a Mox Emerald both make `[Green]` but
        // tapping one over the other is a genuine choice).
        let sigs: Vec<(u8, Vec<Color>)> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == player
                    && !c.tapped
                    && c.definition
                        .activated_abilities
                        .iter()
                        .any(|a| is_mana_ability(&a.effect))
            })
            .map(source_kind_signature)
            .collect();

        // Reserve sources for each still-needed colored pip (most dedicated
        // first). A colour with two *different* source types able to make it
        // is a real choice.
        let mut reserved = vec![false; sigs.len()];
        let mut color_choice = false;
        for (c, rc) in &colored_from_sources {
            let mut cands: Vec<usize> = (0..sigs.len())
                .filter(|i| !reserved[*i] && sigs[*i].1.contains(c))
                .collect();
            if (cands.len() as u32) < *rc {
                return false; // unaffordable for this colour
            }
            let distinct: HashSet<&(u8, Vec<Color>)> = cands.iter().map(|i| &sigs[*i]).collect();
            if distinct.len() > 1 {
                color_choice = true;
            }
            cands.sort_by_key(|i| sigs[*i].1.len()); // dedicated (shortest sig) first
            for &i in cands.iter().take(*rc as usize) {
                reserved[i] = true;
            }
        }

        // Generic: pool leftover first, then the remaining untapped sources.
        let remaining: Vec<&(u8, Vec<Color>)> =
            (0..sigs.len()).filter(|i| !reserved[*i]).map(|i| &sigs[i]).collect();
        let pool_left = pool.total().saturating_sub(pool_used);
        let gen_from_sources = generic.saturating_sub(pool_left);
        if (remaining.len() as u32) < gen_from_sources {
            return false; // unaffordable
        }
        if color_choice {
            return true;
        }
        if gen_from_sources == 0 {
            return false; // only forced colored taps remain — auto-tap them
        }
        // More candidate sources than the generic needs → some are held
        // back; that's a real choice only if they aren't all interchangeable.
        if (remaining.len() as u32) > gen_from_sources {
            let distinct: HashSet<&&(u8, Vec<Color>)> = remaining.iter().collect();
            return distinct.len() >= 2;
        }
        false
    }

    /// Ceiling for the "choose X" prompt on an {X} cost: floating mana plus
    /// one per untapped mana source, minus the cost's fixed part. Display /
    /// clamp guidance only — the payment path remains the authority on
    /// whether the chosen X is actually affordable.
    fn max_prompt_x(&self, player: usize, cost: &crate::mana::ManaCost) -> u32 {
        let pool = self.players[player].mana_pool.total();
        let sources = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == player
                    && !c.tapped
                    && (c.definition
                        .activated_abilities
                        .iter()
                        .any(|a| is_mana_ability(&a.effect))
                        || !self.intrinsic_land_mana_abilities(c.id).is_empty())
            })
            .count() as u32;
        let fixed = cost.with_x_value(0).cmc();
        (pool + sources).saturating_sub(fixed)
    }

    /// True when `player` holds a hand card with a live from-hand mana
    /// ability (Elvish / Simian Spirit Guide) that could contribute to
    /// `cost` — a payment option the auto-tapper can't see, so the player
    /// must choose manually.
    fn hand_mana_source_could_pay(&self, player: usize, cost: &crate::mana::ManaCost) -> bool {
        use crate::mana::ManaSymbol;
        let flexible = cost.symbols.iter().any(|s| {
            matches!(s, ManaSymbol::Generic(n) if *n > 0)
                || matches!(s, ManaSymbol::MonoHybrid(_, _))
        });
        let cost_colors = cost.colors();
        self.players[player].hand.iter().any(|c| {
            c.definition.activated_abilities.iter().any(|a| {
                a.from_hand
                    && is_mana_ability(&a.effect)
                    && (flexible
                        || cost_colors
                            .iter()
                            .any(|col| effect_produces_color(&a.effect, *col)))
            })
        })
    }

    // ── Auto-tap mana sources ─────────────────────────────────────────────────

    /// Tap untapped mana sources to cover `cost` for `player`, returning the
    /// events produced. Called before spell/ability payment so the client
    /// doesn't need to manually tap lands before casting.
    ///
    /// `activate_ability` uses `priority.player_with_priority` for permission
    /// checks, but auto-tap may run in contexts where priority is held by
    /// another player (e.g. resolving a Pact upkeep trigger during the
    /// caster's upkeep). We temporarily override priority to `player` so
    /// our `activate_ability` calls don't reject the tap.
    /// Count untapped permanents `player` controls that can tap for
    /// `color` — used by `auto_tap_for_cost_inner` to decide which half
    /// of a hybrid pip is actually producible. Counts an "add any one
    /// color" source (Birds of Paradise, etc.) toward every color, since
    /// the tap loop can script it to the needed color.
    fn untapped_producers_of(&self, player: usize, color: ManaColor) -> u32 {
        self.with_frozen_layers(|g| g.untapped_producers_of_inner(player, color))
    }

    fn untapped_producers_of_inner(&self, player: usize, color: ManaColor) -> u32 {
        let scan = self.grant_scan();
        self.battlefield
            .iter()
            .filter(|c| {
                c.controller == player
                    && !c.tapped
                    && self.effective_mana_abilities_with(c.id, &scan).iter().any(|(_, a)| {
                        effect_produces_color(&a.effect, color)
                    })
            })
            .count() as u32
    }

    pub fn auto_tap_for_cost(&mut self, player: usize, cost: &crate::mana::ManaCost) -> Vec<GameEvent> {
        self.auto_tap_for_cost_filtered(player, cost, false)
    }

    /// `auto_tap_for_cost` with a CR 106.6b source filter: when
    /// `creature_only`, only creature mana sources are tapped, so a
    /// "spend only mana produced by creatures" cast (Myr Superion) never
    /// strands itself by tapping lands it can't spend.
    pub fn auto_tap_for_cost_filtered(
        &mut self,
        player: usize,
        cost: &crate::mana::ManaCost,
        creature_only: bool,
    ) -> Vec<GameEvent> {
        let prev_priority = self.priority.player_with_priority;
        self.priority.player_with_priority = player;
        let events = self.auto_tap_for_cost_inner(player, cost, creature_only);
        self.priority.player_with_priority = prev_priority;
        events
    }

    /// Preference rank for an auto-tap mana source: lower is tapped first.
    /// Plain "{T}: add mana" sources cost nothing but the tap; life and
    /// self-consuming sources (painlands, Lotus Petal, Chromatic Star) are
    /// held back so a generic pip never eats a one-shot artifact or a
    /// chunk of life while an ordinary land sits untapped.
    fn mana_source_cost_rank(a: &crate::effect::ActivatedAbility) -> u8 {
        if a.sac_cost || a.exile_self_cost || a.discard_cost.is_some() {
            2
        } else if a.life_cost > 0 || a.energy_cost > 0 {
            1
        } else {
            0
        }
    }

    /// One pass over the battlefield collecting every untapped mana
    /// source this player controls, with the colours each can make.
    ///
    /// Built once per auto-tap and consulted from there on. The first
    /// version of the smart-tap selection called
    /// `effective_mana_abilities` inside the per-pip selection loop, once
    /// per candidate per colour — O(pips × sources × colours) *engine*
    /// calls, each of which walks the layer system. That is invisible in
    /// a two-player 40-card game and quadratic death in 4-player
    /// Commander: it took `bot_vs_bot_commander_demo_terminates` from
    /// seconds to past its 600 s timeout. Colours can't change from
    /// tapping, so there is no reason to recompute them per pip.
    /// One WUBRG mask per untapped mana source `player` controls, saying
    /// which colours that source can make.
    ///
    /// Exposed for the net encoder, which needs "could this card in hand
    /// be cast right now" as a *feature* and cannot mutate the game to
    /// find out. `mana_source_table` itself stays private: its ordering
    /// and redundancy fields are auto-tap's business, and callers outside
    /// this module have no use for them.
    pub fn untapped_mana_colors(&self, player: usize) -> Vec<[bool; 5]> {
        self.mana_source_table(player, false)
            .into_iter()
            .map(|s| {
                let mut mask = [false; 5];
                for col in s.colors {
                    mask[color_index(col)] = true;
                }
                mask
            })
            .collect()
    }

    /// Frozen: every untapped permanent asks `effective_mana_abilities`, and
    /// each of those runs `printed_land_mana_ability_lost_with` and
    /// `intrinsic_land_mana_abilities`, both of which take a layer pass. One
    /// scope makes the whole table share one gather and one
    /// `ComputedPermanent` per card.
    fn mana_source_table(&self, player: usize, creature_only: bool) -> Vec<ManaSourceInfo> {
        self.with_frozen_layers(|g| g.mana_source_table_inner(player, creature_only))
    }

    fn mana_source_table_inner(&self, player: usize, creature_only: bool) -> Vec<ManaSourceInfo> {
        // One board-level grant scan for the whole table instead of one per
        // untapped permanent (see `grant_scan`).
        let scan = self.grant_scan();
        self.battlefield
            .iter()
            .filter(|c| c.controller == player && !c.tapped)
            .filter(|c| !creature_only || self.permanent_is_creature(c.id))
            .filter_map(|c| {
                let abilities = self.effective_mana_abilities_with(c.id, &scan);
                let (first_idx, first) = abilities.first()?;
                let mut colors = crate::mana::ColorSet::empty();
                let mut color_idx = [0usize; 5];
                for col in ManaColor::ALL {
                    if let Some((i, _)) =
                        abilities.iter().find(|(_, a)| effect_produces_color(&a.effect, col))
                    {
                        colors.insert(col);
                        color_idx[color_index(col)] = *i;
                    }
                }
                Some(ManaSourceInfo {
                    id: c.id,
                    first_idx: *first_idx,
                    rank: Self::mana_source_cost_rank(first),
                    colors,
                    color_idx,
                })
            })
            .collect()
    }

    fn auto_tap_for_cost_inner(
        &mut self,
        player: usize,
        cost: &crate::mana::ManaCost,
        creature_only: bool,
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();
        // Off, the two selection loops below fall back to "cheapest
        // activation, then battlefield order" — the historical behaviour,
        // kept as the ladder control.
        let smart = self.players[player].smart_tap;
        // Built once; the selection loops below re-check `tapped` live but
        // read colours and costs from here. See `mana_source_table`.
        let sources = self.mana_source_table(player, creature_only);

        // Deduct what the pool already covers before deciding what to tap.
        // We track a "virtual" pool snapshot so we don't mutate the real pool here.
        let pool = &self.players[player].mana_pool;
        // Five fixed keys indexed by `color_index`, not a hash table: this
        // runs once per auto-tap and every cost symbol probed it.
        let mut avail = [0u32; 5];
        for c in ManaColor::ALL {
            avail[color_index(c)] = pool.amount(c);
        }
        let mut avail_colorless = pool.colorless_amount();

        let mut still_need_colors: Vec<ManaColor> = Vec::new();
        // Hybrid pips are resolved after the fixed-color pass so the
        // pool is drained by the more-constrained colored pips first.
        let mut hybrids: Vec<(ManaColor, ManaColor)> = Vec::new();
        let mut generic: u32 = 0;

        for sym in &cost.symbols {
            match sym {
                ManaSymbol::Colored(c) => {
                    let have = &mut avail[color_index(*c)];
                    if *have > 0 { *have -= 1; } else { still_need_colors.push(*c); }
                }
                ManaSymbol::Hybrid(a, b) => hybrids.push((*a, *b)),
                ManaSymbol::Phyrexian(c) => {
                    // Pool covers it if available; otherwise paid with life — no tapping.
                    let have = &mut avail[color_index(*c)];
                    if *have > 0 { *have -= 1; }
                }
                ManaSymbol::PhyrexianHybrid(a, b) => {
                    // Either color from the pool; otherwise paid with life.
                    let have_a = &mut avail[color_index(*a)];
                    if *have_a > 0 {
                        *have_a -= 1;
                    } else {
                        let have_b = &mut avail[color_index(*b)];
                        if *have_b > 0 { *have_b -= 1; }
                    }
                }
                ManaSymbol::MonoHybrid(n, c) => {
                    // {n/C}: spend a matching colored mana if on hand;
                    // otherwise treat the pip as {n} generic to tap for.
                    let have = &mut avail[color_index(*c)];
                    if *have > 0 { *have -= 1; } else { generic += n; }
                }
                ManaSymbol::Generic(n) => generic += n,
                ManaSymbol::Colorless(n) => {
                    // {C} must be paid from the colorless bucket.
                    avail_colorless = avail_colorless.saturating_sub(*n);
                    // If colorless bucket can't cover it fully, we'd need to tap a colorless
                    // source — skip that complexity for now (generic fallback handles it).
                }
                ManaSymbol::Snow | ManaSymbol::X => {}
            }
        }

        // Resolve hybrid pips ({a/b}). Each can be paid by one mana of
        // either color — from the pool, or by tapping a source that makes
        // it. Resolve "forced" pips (only one color reachable) first so a
        // limited board isn't spent on the wrong half: {W/B}{W/B} with a
        // Plains + a Swamp must split W and B, and {W/B} with only a Swamp
        // must tap the Swamp rather than hunting for a white source (the
        // previous code always tried color A and stranded the cast).
        // `reach` = pool mana + untapped sources that can make the color.
        fn reach(c: ManaColor, avail: &[u32; 5], prod: &[u32; 5]) -> u32 {
            avail[color_index(c)] + prod[color_index(c)]
        }
        if !hybrids.is_empty() {
            let mut prod = [0u32; 5];
            for c in ManaColor::ALL {
                prod[color_index(c)] = self.untapped_producers_of(player, c);
            }
            while !hybrids.is_empty() {
                let idx = hybrids
                    .iter()
                    .position(|(a, b)| (reach(*a, &avail, &prod) > 0) ^ (reach(*b, &avail, &prod) > 0))
                    .or_else(|| {
                        hybrids.iter().position(|(a, b)| {
                            reach(*a, &avail, &prod) > 0 || reach(*b, &avail, &prod) > 0
                        })
                    })
                    .unwrap_or(0);
                let (a, b) = hybrids.remove(idx);
                let pick = if reach(a, &avail, &prod) > 0 { a } else { b };
                if avail[color_index(pick)] > 0 {
                    // Already in the pool — consume it, no tapping needed.
                    avail[color_index(pick)] -= 1;
                } else if prod[color_index(pick)] > 0 {
                    // Reserve an untapped source of this color to tap below.
                    prod[color_index(pick)] -= 1;
                    still_need_colors.push(pick);
                } else {
                    // Neither color reachable — push anyway; the tap loop
                    // no-ops and the cast fails downstream as before.
                    still_need_colors.push(pick);
                }
            }
        }

        // Remaining pool total after colored deductions covers generic pips.
        let pool_total_left: u32 = avail.iter().sum::<u32>() + avail_colorless;
        let generic_to_tap = generic.saturating_sub(pool_total_left);

        // Tap a color-matched source for each still-needed colored pip.
        // For abilities that produce `AnyOneColor` (Black Lotus, Birds of
        // Paradise, Mox Diamond, etc.) the source's own resolver asks the
        // installed `Decider` which color to add. We temporarily swap in a
        // `ScriptedDecider` that answers with `color`, so the chosen color
        // matches the pip we're trying to satisfy. (Without this, the
        // default `AutoDecider` always picks White and leaves the requested
        // color unfilled.)
        for color in still_need_colors {
            // `controller` not `owner`: a permanent you've stolen
            // (Threaten / Mind Control) is a tap-for-mana source for
            // you, regardless of its original ownership — `sources` is
            // filtered on controller for the same reason.
            //
            // Among sources that make this colour, spend the *least*
            // flexible one: a Swamp pays {B} before a Dimir dual does,
            // leaving the dual free for whichever colour comes up next.
            // Same reasoning as the "dedicated first" reservation in
            // `manual_tap_is_a_real_choice`. Cheapest first, then
            // narrowest; `min_by_key` keeps the first of equal keys, so
            // battlefield order still breaks remaining ties as before.
            let source = sources
                .iter()
                .filter(|s| !self.battlefield_find(s.id).is_some_and(|c| c.tapped))
                .filter_map(|s| {
                    let idx = s.colors.contains(color).then(|| s.color_idx[color_index(color)])?;
                    let breadth = if smart { s.colors.len() } else { 0 };
                    Some((s.rank, breadth, s.id, idx))
                })
                .min_by_key(|&(rank, breadth, ..)| (rank, breadth))
                .map(|(_, _, id, idx)| (id, idx));
            if let Some((id, idx)) = source {
                let scripted = crate::decision::ScriptedDecider::new([
                    crate::decision::DecisionAnswer::Color(color),
                ]);
                let prev_decider = std::mem::replace(
                    &mut self.decider,
                    Box::new(scripted),
                );
                // Force synchronous resolution: if the player normally wants
                // a UI prompt for `AnyOneColor`, auto-tap must still finish
                // inline (otherwise the cast aborts mid-payment with a
                // pending decision). The scripted decider already supplies
                // the right answer.
                let prev_wants_ui = self.players[player].wants_ui;
                self.players[player].wants_ui = false;
                let result = self.activate_ability(id, idx, None, Vec::new(), None, None);
                self.decider = prev_decider;
                self.players[player].wants_ui = prev_wants_ui;
                if let Ok(mut evs) = result {
                    events.append(&mut evs);
                }
            }
        }

        // Tap any mana source for remaining generic pips, spending the
        // most replaceable ones first — see `source_redundancy`.
        //
        // `redundancy` reads `sources`, which this loop never mutates
        // (tapping writes the battlefield, and the table's colours can't
        // change from a tap — that is why it is built once at all), so the
        // ranking is constant across the pips and is computed once here.
        // It used to be recomputed per live source per pip. Gated on there
        // being a generic pip at all: most costs have none, and building the
        // ranking is O(sources² × colours).
        let keep_by_idx: Vec<u32> = if smart && generic_to_tap > 0 {
            sources.iter().map(|s| s.redundancy(&sources)).collect()
        } else {
            Vec::new()
        };
        for _ in 0..generic_to_tap {
            // Same controller-vs-owner fix as the colored-pip loop.
            let source = sources
                .iter()
                .enumerate()
                .filter(|(_, s)| !self.battlefield_find(s.id).is_some_and(|c| c.tapped))
                .map(|(i, s)| {
                    let keep = if smart { keep_by_idx[i] } else { 0 };
                    (s.rank, std::cmp::Reverse(keep), s.id, s.first_idx)
                })
                .min_by_key(|&(rank, keep, ..)| (rank, keep))
                .map(|(_, _, id, idx)| (id, idx));
            let Some((id, idx)) = source else { break };
            if let Ok(mut evs) = self.activate_ability(id, idx, None, Vec::new(), None, None) {
                events.append(&mut evs);
            } else {
                break;
            }
        }

        events
    }

    // ── Activate ability ──────────────────────────────────────────────────────

    /// Collect the activated abilities granted to the permanent `card_id`
    /// by `StaticEffect::GrantActivatedAbility` statics in play (Galazeth
    /// Prismari, Cryptolith Rite). Each grant's `applies_to` filter is
    /// evaluated from the static source's controller so "you control"
    /// clauses scope to that player's permanents. Returned in battlefield
    /// order so a permanent's granted-ability indices are stable within a
    /// recompute. Surfaced by `activate_ability` at indices ≥ the
    /// permanent's printed-ability count.
    /// Intrinsic basic-land-type mana abilities for a battlefield
    /// permanent, derived from its *computed* land types (CR 305.6): any
    /// land with a basic land type has the intrinsic `{T}: Add <color>`.
    /// We derive only for basic land types the permanent gained via a
    /// continuous effect (computed but not printed), so printed basics —
    /// which hard-code `tap_add` — aren't double-counted. Lets Blood Moon
    /// / Spreading Seas / Urborg-style type changers tap for the right
    /// colour. Surfaced at ability indices after printed + granted.
    pub(crate) fn intrinsic_land_mana_abilities(
        &self,
        card_id: CardId,
    ) -> Vec<crate::effect::ActivatedAbility> {
        let Some(card) = self.battlefield_find(card_id) else {
            return vec![];
        };
        let Some(computed) = self.computed_permanent(card_id) else {
            return vec![];
        };
        Self::intrinsic_land_mana_abilities_with(card, &computed.subtypes.land_types)
    }

    /// [`intrinsic_land_mana_abilities`](Self::intrinsic_land_mana_abilities)
    /// against a computed land-type list the caller already holds.
    /// `effective_mana_abilities_with` reads the same card's computed types
    /// for its printed-ability check, and a `computed_permanent` outside a
    /// freeze scope is a whole-game gather.
    pub(crate) fn intrinsic_land_mana_abilities_with(
        card: &crate::card::CardInstance,
        computed_land_types: &[crate::card::LandType],
    ) -> Vec<crate::effect::ActivatedAbility> {
        use crate::card::LandType;
        let printed: &[LandType] = &card.definition.subtypes.land_types;
        let mut out = Vec::new();
        for lt in computed_land_types {
            if printed.contains(lt) {
                continue;
            }
            let color = match lt {
                LandType::Plains => ManaColor::White,
                LandType::Island => ManaColor::Blue,
                LandType::Swamp => ManaColor::Black,
                LandType::Mountain => ManaColor::Red,
                LandType::Forest => ManaColor::Green,
                _ => continue,
            };
            out.push(crate::effect::ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                effect: Effect::AddMana {
                    who: crate::effect::PlayerRef::You,
                    pool: ManaPayload::Colors(vec![color]),
                },
                ..Default::default()
            });
        }
        out
    }

    /// CR 305.6 / CR 612 — a basic land's printed `{T}: Add <color>` is really
    /// the intrinsic ability its type line grants, so it goes away once a type
    /// rewrite takes that basic type off the *computed* type line (Spreading
    /// Seas, Blood Moon, a Trait Doctoring color/land-word change). Printed
    /// mana abilities that aren't a basic type's intrinsic ability are real
    /// rules text and survive.
    ///
    /// Takes the computed land-type list the caller already holds — every
    /// caller is inside a scope that has one, so the `&self` variant that
    /// fetched its own (a whole-game gather per call) is gone. See
    /// [`intrinsic_land_mana_abilities_with`](Self::intrinsic_land_mana_abilities_with).
    pub(crate) fn printed_land_mana_ability_lost_with(
        card: &crate::card::CardInstance,
        index: usize,
        computed_land_types: &[crate::card::LandType],
    ) -> bool {
        use crate::card::LandType;
        if !card.definition.is_land() {
            return false;
        }
        let Some(ability) = card.definition.activated_abilities.get(index) else { return false };
        let color = match &ability.effect {
            Effect::AddMana { pool: ManaPayload::Colors(cs), .. } if cs.len() == 1 => cs[0],
            _ => return false,
        };
        let basic = match color {
            ManaColor::White => LandType::Plains,
            ManaColor::Blue => LandType::Island,
            ManaColor::Black => LandType::Swamp,
            ManaColor::Red => LandType::Mountain,
            ManaColor::Green => LandType::Forest,
        };
        if !card.definition.subtypes.land_types.contains(&basic) {
            return false;
        }
        !computed_land_types.contains(&basic)
    }

    /// `(index, ability)` for every mana-producing activated ability a
    /// battlefield permanent can currently use — printed, granted, and
    /// intrinsic basic-land — in `activate_ability`'s index order. The
    /// single source of truth for the auto-tap source finders so a land
    /// whose type changed (Spreading Seas / Blood Moon / Urborg) taps for
    /// its computed colours.
    pub fn effective_mana_abilities(
        &self,
        card_id: CardId,
    ) -> Vec<(usize, std::borrow::Cow<'_, crate::effect::ActivatedAbility>)> {
        self.effective_mana_abilities_with(card_id, &self.grant_scan())
    }

    /// [`effective_mana_abilities`](Self::effective_mana_abilities) against a
    /// prebuilt [`grant_scan`](Self::grant_scan) — `mana_source_table_inner`
    /// asks this per untapped permanent, so the scan is built once per table.
    pub(crate) fn effective_mana_abilities_with(
        &self,
        card_id: CardId,
        scan: &GrantScan<'_>,
    ) -> Vec<(usize, std::borrow::Cow<'_, crate::effect::ActivatedAbility>)> {
        use std::borrow::Cow;
        let Some(card) = self.battlefield_find(card_id) else {
            return vec![];
        };
        let printed_count = card.definition.activated_abilities.len();
        // One layer read for the whole list. Both CR 305.6 checks below ask
        // the same card for the same thing — its *computed* land types — and
        // a `computed_permanent` outside a freeze scope gathers every
        // continuous effect in the game. This was one gather per printed mana
        // ability plus one more for the intrinsic pass.
        //
        // And the layer pass is skipped outright when nothing in scope can
        // rewrite a land type: exactly three modifications write
        // `subtypes.land_types` (see `rewrites_land_types`), and with none of
        // them present the computed type line *is* the printed one. Inside a
        // freeze scope the effect list is already gathered, so the test is a
        // walk of it; outside one, `frozen_effects` returns `None` and the
        // old path stands rather than paying a gather to save a gather.
        let computed = match self.frozen_effects() {
            Some(fx) if !fx.iter().any(rewrites_land_types) => None,
            _ => self.computed_permanent(card_id),
        };
        let computed_land_types: &[crate::card::LandType] = match &computed {
            // `card` came out of `battlefield_find`, so `computed_permanent`
            // found it too — a `None` here is the gate above, not a miss.
            Some(cp) => &cp.subtypes.land_types,
            None => &card.definition.subtypes.land_types,
        };
        // The printed abilities are borrowed from `card.definition`, which
        // outlives the call — only `granted_abilities_for` and
        // `intrinsic_land_mana_abilities` synthesize, and all three callers
        // read `.effect` and the cost fields. Cloning them was one
        // `ActivatedAbility` deep copy per printed mana ability per untapped
        // permanent per `auto_tap_for_cost_inner`.
        let mut out: Vec<(usize, Cow<'_, crate::effect::ActivatedAbility>)> = Vec::new();
        for (i, a) in card.definition.activated_abilities.iter().enumerate() {
            if is_mana_ability(&a.effect)
                && !Self::printed_land_mana_ability_lost_with(card, i, computed_land_types)
            {
                out.push((i, Cow::Borrowed(a)));
            }
        }
        let granted = self.granted_abilities_with(card_id, scan);
        let gc = granted.len();
        for (j, a) in granted.into_iter().enumerate() {
            if is_mana_ability(&a.effect) {
                out.push((printed_count + j, Cow::Owned(a)));
            }
        }
        for (k, a) in Self::intrinsic_land_mana_abilities_with(card, computed_land_types)
            .into_iter()
            .enumerate()
        {
            out.push((printed_count + gc + k, Cow::Owned(a)));
        }
        out
    }

    /// CR 702.61a — true while any spell on the stack has split second.
    pub(crate) fn stack_has_split_second(&self) -> bool {
        self.stack.iter().any(|si| match si {
            crate::game::types::StackItem::Spell { card, .. } => {
                card.definition.keywords.contains(&crate::card::Keyword::SplitSecond)
            }
            _ => false,
        })
    }

    /// CR 702.61 — which actions a split-second lock forbids: every cast,
    /// loyalty activations, and activated abilities that aren't mana
    /// abilities (keyword activations like Cycling / Equip / Crew /
    /// Ninjutsu are activated abilities too, CR 702.29f). Special actions
    /// (land drops, Foretell, Plot, Suspend, TurnFaceUp, CompanionToHand,
    /// UnlockRoomDoor) and decision submissions stay legal (702.61b).
    pub(crate) fn split_second_blocks(&self, action: &GameAction) -> bool {
        use GameAction as A;
        match action {
            a if a.is_cast() => true,
            A::ActivateLoyaltyAbility { .. }
            | A::Cycle { .. }
            | A::Landcycle { .. }
            | A::Reinforce { .. }
            | A::ActivateDiscardAbility { .. }
            | A::Equip { .. }
            | A::Reconfigure { .. }
            | A::Crew { .. }
            | A::Saddle { .. }
            | A::Ninjutsu { .. } => true,
            A::ActivateAbility { card_id, ability_index, .. } => {
                // Allow mana abilities (CR 702.61b). Resolve the ability the
                // same way `activate_ability` does: printed first (any zone),
                // then granted abilities at indices ≥ printed_count.
                let printed = self
                    .battlefield_find(*card_id)
                    .or_else(|| {
                        self.players.iter().find_map(|p| {
                            p.graveyard.iter().chain(p.hand.iter()).find(|c| c.id == *card_id)
                        })
                    })
                    .and_then(|c| c.definition.activated_abilities.get(*ability_index).cloned());
                let ability = printed.or_else(|| {
                    let printed_count = self
                        .find_card_anywhere(*card_id)
                        .map(|c| c.definition.activated_abilities.len())?;
                    self.granted_abilities_for(*card_id)
                        .into_iter()
                        .nth(ability_index.checked_sub(printed_count)?)
                });
                !ability.is_some_and(|a| is_mana_ability(&a.effect))
            }
            _ => false,
        }
    }

    /// The board-level half of [`granted_abilities_for`]: which grant sources
    /// are live right now, independent of which permanent is asking. Building
    /// it costs three whole-battlefield walks and both graveyards — the same
    /// walks [`granted_abilities_for`] used to run *per call*, from three
    /// per-card loops (`bot::usable_abilities`, `effective_mana_abilities`,
    /// `bot::available_mana`). Build one per loop and hand it to
    /// [`granted_abilities_with`](Self::granted_abilities_with).
    ///
    /// Bound to `&self`, so it is invalid the moment the board changes; there
    /// is deliberately no cached copy on `GameState`.
    pub(crate) fn grant_scan(&self) -> GrantScan<'_> {
        use crate::effect::{Selector, StaticEffect};
        let mut scan = GrantScan::default();
        // CR 315.5 — a face-up conspiracy grants from the command zone too.
        for src in self.all_static_sources() {
            for sa in &src.definition.static_abilities {
                // CR 611.2 — a grant may sit under a duration/predicate
                // wrapper ("Threshold — this creature has '…'"); unwrap it
                // and honour the gate.
                let Some(inner) = self.active_static(&sa.effect, src) else { continue };
                let StaticEffect::GrantActivatedAbility { applies_to, ability, condition } = inner
                else {
                    continue;
                };
                // Hellbent-style gate: the grant is live only while the
                // source's controller satisfies `condition`. Depends on the
                // source, not on the permanent asking, so it is decided here.
                if let Some(pred) = condition {
                    let cond_ctx = crate::game::effects::EffectContext::for_ability(
                        src.id, src.controller, None,
                    );
                    if !self.evaluate_predicate(pred, &cond_ctx) {
                        continue;
                    }
                }
                scan.statics.push((applies_to, ability, src));
            }
        }
        // Riftstone Portal — "as long as this card is in your graveyard,
        // lands you control have '…'". The grant is live from the graveyard,
        // scoped to the owning seat's permanents.
        for (seat, pl) in self.players.iter().enumerate() {
            for src in pl.graveyard.iter() {
                for sa in &src.definition.static_abilities {
                    let StaticEffect::GrantActivatedAbilityFromGraveyard { applies_to, ability } =
                        &sa.effect
                    else {
                        continue;
                    };
                    let Selector::EachPermanent(req) = applies_to else { continue };
                    scan.graveyard.push((req, &**ability, seat, src.id));
                }
            }
        }
        for src in self.battlefield.iter() {
            // CR 702.95 — Soulbond-granted activated abilities (Deadeye
            // Navigator's flicker). A paired creature carrying a
            // `soulbond_bonus` with `activated_abilities` grants them to BOTH
            // itself and its partner.
            if let Some(bonus) = &src.definition.soulbond_bonus
                && !bonus.activated_abilities.is_empty()
                && let Some(partner) = src.soulbond_partner
                && self.battlefield.iter().any(|c| c.id == partner)
            {
                scan.soulbond.push((src.id, partner, &bonus.activated_abilities));
            }
            // CR 702.6e — Equipment/Aura-granted activated abilities, matched
            // per card by `attached_to`.
            if src.attached_to.is_some() && src.definition.equipped_bonus.is_some() {
                scan.equipment.push(src);
            }
        }
        scan
    }

    pub fn granted_abilities_for(
        &self,
        card_id: CardId,
    ) -> Vec<crate::effect::ActivatedAbility> {
        self.granted_abilities_with(card_id, &self.grant_scan())
    }

    /// [`granted_abilities_for`](Self::granted_abilities_for) against a
    /// prebuilt [`grant_scan`](Self::grant_scan).
    pub(crate) fn granted_abilities_with(
        &self,
        card_id: CardId,
        scan: &GrantScan<'_>,
    ) -> Vec<crate::effect::ActivatedAbility> {
        use crate::effect::{Selector, StaticEffect};
        let tgt = Target::Permanent(card_id);
        let mut out = Vec::new();
        // One lookup, reused by every `me`-reading block below: this is called
        // ~272 k times per six bot games and `battlefield_find` is a linear
        // scan of the whole battlefield.
        let Some(me) = self.battlefield_find(card_id) else {
            // A card outside the battlefield can still carry an instance grant
            // — Cursecloth Wrappings hands a graveyard creature card embalm.
            if let Some(c) = self.find_card_anywhere(card_id) {
                out.extend(c.granted_activated_abilities.iter().cloned());
                out.extend(c.granted_activated_eot.iter().cloned());
            }
            return out;
        };
        // Which of the "has the activated abilities of …" statics this
        // permanent carries, in one pass of its static abilities instead of
        // one pass per block below.
        let (mut welder, mut ooze, mut marvin, mut kraj, mut safehouse) =
            (false, false, false, false, false);
        for sa in &me.definition.static_abilities {
            match sa.effect {
                StaticEffect::HasActivatedAbilitiesOfExiledWithSelf => welder = true,
                StaticEffect::HasActivatedAbilitiesOfGraveyardCreatures => ooze = true,
                StaticEffect::HasActivatedAbilitiesOfOtherNamedControlledCreatures => {
                    marvin = true
                }
                StaticEffect::HasActivatedAbilitiesOfCounteredCreatures => kraj = true,
                StaticEffect::HasActivatedAbilitiesOfGraveyardLands => safehouse = true,
                _ => {}
            }
        }
        // Instance-granted abilities first (Urza's Saga chapters) — the
        // client view lists printed + instance-granted in this order, so
        // their indices must come before the battlefield-static grants.
        out.extend(me.granted_activated_abilities.iter().cloned());
        out.extend(me.granted_activated_eot.iter().cloned());
        // Myr Welder — "has all activated abilities of all cards exiled
        // with it", read live off the imprint pile.
        if welder {
            for imp in self.exile.iter().filter(|e| e.exiled_with == Some(card_id)) {
                out.extend(imp.definition.activated_abilities.iter().cloned());
            }
        }
        // CR 804.2 — the deploy creatures option gives every creature
        // "{T}: Target teammate gains control of this creature. Activate
        // only as a sorcery."
        if self.deploy_creatures && me.definition.is_creature() {
            out.push(crate::effect::ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::GainControl {
                    what: Selector::This,
                    to: Some(crate::effect::PlayerRef::EachTeammate),
                    duration: crate::effect::Duration::Permanent,
                },
                ..Default::default()
            });
        }
        // CR 315.5 — battlefield and command-zone `GrantActivatedAbility`
        // statics that are already known live (see `grant_scan`).
        for (applies_to, ability, src) in &scan.statics {
            match applies_to {
                Selector::EachPermanent(req) => {
                    // Evaluate the filter from the granting source's
                    // controller so "ControlledByYou" picks that
                    // player's permanents (and `NamedBySource` reads
                    // the granting source's chosen name).
                    if self.evaluate_requirement_static(req, &tgt, src.controller, Some(src.id)) {
                        out.push((*ability).clone());
                    }
                }
                // "Enchanted/equipped creature has [ability]" — the
                // grant rides the attachment link (Splinter Twin).
                Selector::AttachedTo(inner) if matches!(**inner, Selector::This) => {
                    if src.attached_to == Some(card_id) {
                        out.push((*ability).clone());
                    }
                }
                // Self-grant — "this creature has '[ability]'", optionally
                // condition-gated (Gobhobbler Rats' Hellbent regenerate).
                Selector::This => {
                    if src.id == card_id {
                        out.push((*ability).clone());
                    }
                }
                _ => continue,
            }
        }
        // Riftstone Portal — "as long as this card is in your graveyard,
        // lands you control have '…'". The grant is live from the graveyard,
        // scoped to the owning seat's permanents.
        for (req, ability, seat, src_id) in &scan.graveyard {
            if self.evaluate_requirement_static(req, &tgt, *seat, Some(*src_id)) {
                out.push((*ability).clone());
            }
        }
        // Necrotic Ooze — a permanent with
        // `HasActivatedAbilitiesOfGraveyardCreatures` gains every battlefield-
        // usable activated ability of every creature card in every graveyard.
        // Skip graveyard-only activations (`from_graveyard` / `exile_self_cost`)
        // — those function only from the graveyard, so the Ooze can't use them.
        if ooze {
            for pl in &self.players {
                for card in &pl.graveyard {
                    if !card.definition.is_creature() {
                        continue;
                    }
                    for ab in &card.definition.activated_abilities {
                        if ab.from_graveyard || ab.exile_self_cost {
                            continue;
                        }
                        out.push(ab.clone());
                    }
                }
            }
        }
        // Marvin, Murderous Mimic — every activated ability of each creature
        // its controller controls whose name differs from Marvin's.
        if marvin {
            let (seat, name) = (me.controller, me.definition.name);
            for other in &self.battlefield {
                if other.controller != seat
                    || other.definition.name == name
                    || !other.definition.is_creature()
                {
                    continue;
                }
                out.extend(other.definition.activated_abilities.iter().cloned());
            }
        }
        // Experiment Kraj — every activated ability of each *other* creature
        // carrying a +1/+1 counter.
        if kraj {
            for other in &self.battlefield {
                if other.id == card_id
                    || !other.definition.is_creature()
                    || other.counter_count(crate::card::CounterType::PlusOnePlusOne) == 0
                {
                    continue;
                }
                out.extend(other.definition.activated_abilities.iter().cloned());
            }
        }
        // Mirran Safehouse — every battlefield-usable activated ability of
        // every land card in every graveyard.
        if safehouse {
            for pl in &self.players {
                for card in &pl.graveyard {
                    if !card.definition.is_land() {
                        continue;
                    }
                    for ab in &card.definition.activated_abilities {
                        if ab.from_graveyard || ab.exile_self_cost {
                            continue;
                        }
                        out.push(ab.clone());
                    }
                }
            }
        }
        // Conspicuous Snoop — while the controller's library top matches the
        // static's filter, the source has all of that card's battlefield-
        // usable activated abilities (the top card is revealed by the
        // companion `TopOfLibraryRevealed` static).
        {
            for sa in &me.definition.static_abilities {
                let StaticEffect::HasActivatedAbilitiesOfLibraryTop { filter } = &sa.effect else {
                    continue;
                };
                if let Some(top) = self.players[me.controller].library.first()
                    && self.evaluate_requirement_on_card(filter, top, me.controller)
                {
                    for ab in &top.definition.activated_abilities {
                        if ab.from_graveyard || ab.exile_self_cost || ab.from_hand {
                            continue;
                        }
                        out.push(ab.clone());
                    }
                }
            }
        }
        // Agatha's Soul Cauldron — a creature you control with a +1/+1
        // counter has all activated abilities of creature cards exiled
        // with any Cauldron its controller controls.
        if me.definition.is_creature()
            && me.counter_count(crate::card::CounterType::PlusOnePlusOne) > 0
        {
            let cauldrons: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == me.controller
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(
                                sa.effect,
                                StaticEffect::CounteredCreaturesHaveAbilitiesOfExiledWithSource
                            )
                        })
                })
                .map(|c| c.id)
                .collect();
            for exiled in &self.exile {
                if exiled.definition.is_creature()
                    && exiled.exiled_with.is_some_and(|s| cauldrons.contains(&s))
                {
                    for ab in &exiled.definition.activated_abilities {
                        if ab.from_graveyard || ab.exile_self_cost {
                            continue;
                        }
                        out.push(ab.clone());
                    }
                }
            }
        }
        // CR 702.95 — Soulbond-granted activated abilities (Deadeye Navigator's
        // flicker). A paired creature carrying a `soulbond_bonus` with
        // `activated_abilities` grants them to BOTH itself and its partner.
        for (src_id, partner, abilities) in &scan.soulbond {
            if *src_id == card_id || *partner == card_id {
                out.extend(abilities.iter().cloned());
            }
        }
        // CR 702.6e — Equipment-granted activated abilities. An Equipment whose
        // `equipped_bonus.activated_abilities` is non-empty and is attached to
        // this creature grants them (Wrench's "{3}, {T}: Tap target creature").
        for eq in &scan.equipment {
            if eq.attached_to != Some(card_id) {
                continue;
            }
            if let Some(bonus) = &eq.definition.equipped_bonus {
                out.extend(bonus.activated_abilities.iter().cloned());
                // Host-conditional grants ("as long as enchanted creature is a
                // Wizard, it has …" — Lavamancer's Skill).
                for cond in &bonus.conditional {
                    if cond.activated_abilities.is_empty() {
                        continue;
                    }
                    if self.evaluate_requirement_on_card(&cond.host_filter, me, eq.controller) {
                        out.extend(cond.activated_abilities.iter().cloned());
                    }
                }
            }
        }
        // CR 721.2a — Station `{N+}` activated-ability bands. While the source's
        // charge-counter count meets a band threshold, that band's activated
        // abilities are usable (a Planet's `12+ | {cost}: …`).
        if !me.definition.station.is_empty() {
            let charges = me.counter_count(crate::card::CounterType::Charge);
            for band in me.definition.station.iter().filter(|b| charges >= b.min) {
                out.extend(band.activated.iter().cloned());
            }
        }
        out
    }

    /// Activate an ability whose cost accepts tapped helpers: CR 701.67
    /// Waterbend (`ActivatedAbility.waterbend` — artifacts or creatures, each
    /// paying {1} of the generic) or CR 702.51 convoke on the ability
    /// (`.convoke` — creatures only, each paying one generic *or* one colored
    /// pip of its own color). Floats the helper mana, then defers to
    /// `activate_ability` for the full payment.
    pub(crate) fn activate_ability_waterbend(
        &mut self,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        x_value: Option<u32>,
        helpers: &[CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        // The ability must exist on a battlefield source and be flagged
        // waterbend (artifacts or creatures help) or convoke (creatures only).
        let creatures_only;
        // CR 702.51b — convoke helpers can pay colored pips too, so their cap
        // is the whole cost; waterbend helpers only reach the generic.
        let mut colored_pips: Vec<crate::mana::Color> = Vec::new();
        let generic_total = {
            let Some(c) = self.battlefield.iter().find(|c| c.id == card_id) else {
                return Err(GameError::CardNotOnBattlefield(card_id));
            };
            let Some(ab) = c.definition.activated_abilities.get(ability_index) else {
                return Err(GameError::AbilityIndexOutOfBounds);
            };
            if !ab.waterbend && !ab.convoke {
                return Err(GameError::AbilityIndexOutOfBounds); // no helper-paid cost
            }
            creatures_only = ab.convoke;
            // For a "Waterbend {X}" ability the generic is the chosen X; otherwise
            // it's the printed generic pip total.
            if ab.convoke {
                colored_pips = ab
                    .mana_cost
                    .symbols
                    .iter()
                    .filter_map(|sym| match sym {
                        crate::mana::ManaSymbol::Colored(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
            }
            if ab.mana_cost.has_x() {
                ab.mana_cost.with_x_value(x_value.unwrap_or(0)).generic_total()
            } else {
                ab.mana_cost.generic_total()
            }
        };
        let helper_cap = generic_total + colored_pips.len() as u32;
        if helpers.len() > helper_cap as usize {
            return Err(GameError::SelectionRequirementViolated); // too many helpers
        }
        // Validate helpers up front: untapped artifacts/creatures the activator
        // controls (the source itself is eligible if it isn't tapped by the cost).
        for cid in helpers {
            let ok = self.battlefield.iter().any(|c| {
                c.id == *cid
                    && c.controller == p
                    && !c.tapped
                    && (c.definition.is_creature()
                        || (!creatures_only && c.definition.is_artifact()))
            });
            if !ok {
                return Err(GameError::CardNotOnBattlefield(*cid));
            }
        }
        // Float the helper mana (tap each for {1} colorless), then let
        // activate_ability pay the now-reduced remainder. Skip the float-spend
        // prompt — the colorless was tapped expressly to pay this cost.
        let snapshot = self.snapshot_payment_state(p);
        for cid in helpers {
            let colors = self
                .battlefield_find(*cid)
                .map(|c| c.definition.printed_colors())
                .unwrap_or_default();
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == *cid) {
                c.tapped = true;
            }
            // Prefer a colored pip this creature can actually cover; generic
            // otherwise (CR 702.51b lets the player choose — this takes the
            // pip that would otherwise need coloured mana).
            match colored_pips.iter().position(|c| colors.contains(c)) {
                Some(i) => {
                    let color = colored_pips.remove(i);
                    self.players[p].mana_pool.add(color, 1);
                }
                None => self.players[p].mana_pool.add_colorless(1),
            }
        }
        self.pending_cast_spend_float = Some(true);
        let r = self.activate_ability(card_id, ability_index, target, additional_targets, x_value, None);
        if r.is_err() {
            self.pending_cast_spend_float = None;
            self.restore_payment_state(p, snapshot);
        }
        r
    }

    #[allow(clippy::too_many_arguments)]
    /// CR 106.6b — mana a *creature* produces carries provenance so a
    /// "spend only mana produced by creatures" cost (Myr Superion) can find
    /// it. Tagging the pool delta here catches printed, granted (Cryptolith
    /// Rite) and intrinsic mana abilities in one place.
    pub(crate) fn activate_ability(
        &mut self,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        x_value: Option<u32>,
        chosen_mode: Option<usize>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        self.check_free_activation_loop(card_id, ability_index)?;
        // Mana produced by a *creature* is marked as such (CR 106.12
        // restrictions). Whether the source is one is a layer read, and
        // `_inner` already computes that permanent for the ability lookup —
        // so it hands back the pre-activation pool when the source is a
        // creature, instead of this frame paying a second whole-game gather.
        // Nothing has moved mana at the point it fills this in.
        let mut before: Option<crate::mana::ManaPool> = None;
        let out = self.activate_ability_inner(
            card_id,
            ability_index,
            target,
            additional_targets,
            x_value,
            chosen_mode,
            &mut before,
        );
        if let Some(before) = before {
            let pool = &mut self.players[p].mana_pool;
            for c in ManaColor::ALL {
                let d = pool.amount(c).saturating_sub(before.amount(c));
                if d > 0 {
                    pool.mark_from_creature(Some(c), d);
                }
            }
            let d = pool.colorless_amount().saturating_sub(before.colorless_amount());
            if d > 0 {
                pool.mark_from_creature(None, d);
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn activate_ability_inner(
        &mut self,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        x_value: Option<u32>,
        chosen_mode: Option<usize>,
        // Out: the caller's pre-activation mana pool, filled only when the
        // source is a creature — see `activate_ability`.
        creature_mana_before: &mut Option<crate::mana::ManaPool>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;

        // CR 801.6 — a player can't activate abilities of an object outside
        // their range of influence.
        if !self.object_in_range_of(p, card_id) {
            return Err(GameError::OutOfRange);
        }

        // Consume any "another permanent" cost picks from an
        // `ActivateAbilityChoice` resume up front, so a failure anywhere below
        // can't leak them onto the next activation. `None` on the first attempt
        // (may suspend for the choice); `Some` on the replay (used in lieu of
        // the auto-pick).
        // Abeyance — "that player can't activate abilities that aren't mana
        // abilities" this turn.
        if self.players[p].cant_activate_nonmana_abilities_this_turn
            && let Some(a) = self
                .find_card_anywhere(card_id)
                .and_then(|c| c.definition.activated_abilities.get(ability_index))
            && !is_mana_ability(&a.effect)
        {
            return Err(GameError::AbilityConditionNotMet);
        }

        let chosen_sac_other = self.pending_ability_sac_other.take();
        let chosen_tap_other = self.pending_ability_tap_other.take();
        let chosen_exile_other = self.pending_ability_exile_other.take();
        let chosen_sac_any = self.pending_ability_sac_any.take();
        // CR 601.2g float-spend choice (None until answered; consumed up front
        // so a failure can't leak it onto a later activation).
        let spend_float = self.pending_cast_spend_float.take();

        // Source zone: battlefield by default, the controller's graveyard
        // when the ability is flagged `from_graveyard`, or the controller's
        // hand when flagged `from_hand` (Spirit Guides' exile-to-pitch mana
        // abilities). We scan battlefield first; if missing, fall back to
        // graveyards then hands (any player's; ownership is verified below).
        let (source_in_gy, source_in_hand, source_in_exile, source_in_command, source_owner) = {
            let on_bf = self.battlefield.iter().any(|c| c.id == card_id);
            if on_bf {
                (false, false, false, false, None)
            } else if let Some(o) = self
                .players
                .iter()
                .position(|pl| pl.graveyard.iter().any(|c| c.id == card_id))
            {
                (true, false, false, false, Some(o))
            } else if let Some(o) = self
                .players
                .iter()
                .position(|pl| pl.hand.iter().any(|c| c.id == card_id))
            {
                (false, true, false, false, Some(o))
            } else if let Some(owner) =
                self.exile.iter().find(|c| c.id == card_id).map(|c| c.owner)
            {
                (false, false, true, false, Some(owner))
            // CR 902.5 — a Vanguard's abilities function from the command zone.
            } else if let Some(o) = self
                .players
                .iter()
                .position(|pl| pl.command.iter().any(|c| c.id == card_id))
            {
                (false, false, false, true, Some(o))
            } else {
                return Err(GameError::CardNotOnBattlefield(card_id));
            }
        };

        // This card's computed view, taken once by the battlefield branch
        // below and read again by the CR 602.5 gates further down. Nothing
        // between the two points mutates a layer input (the one write is the
        // `{X}` prompt, which returns), so the second read reuses this
        // instead of taking its own whole-game gather.
        let mut bf_cp = None;
        let ability: crate::effect::ActivatedAbility = if source_in_gy {
            let owner = source_owner.unwrap();
            let card = self.players[owner].graveyard.iter()
                .find(|c| c.id == card_id)
                .ok_or(GameError::AbilityIndexOutOfBounds)?;
            let printed_count = card.definition.activated_abilities.len();
            if ability_index < printed_count {
                card.definition.activated_abilities[ability_index].clone()
            } else {
                // Static-granted graveyard abilities (Varolz's scavenge).
                self.graveyard_granted_abilities(owner, card)
                    .into_iter()
                    .nth(ability_index - printed_count)
                    .ok_or(GameError::AbilityIndexOutOfBounds)?
            }
        } else if source_in_hand {
            let owner = source_owner.unwrap();
            self.players[owner].hand.iter()
                .find(|c| c.id == card_id)
                .and_then(|c| c.definition.activated_abilities.get(ability_index).cloned())
                .ok_or(GameError::AbilityIndexOutOfBounds)?
        } else if source_in_exile {
            self.exile.iter()
                .find(|c| c.id == card_id)
                .and_then(|c| c.definition.activated_abilities.get(ability_index).cloned())
                .ok_or(GameError::AbilityIndexOutOfBounds)?
        } else if source_in_command {
            self.players[source_owner.unwrap()].command.iter()
                .find(|c| c.id == card_id)
                .and_then(|c| c.definition.activated_abilities.get(ability_index).cloned())
                .ok_or(GameError::AbilityIndexOutOfBounds)?
        } else {
            let pos = self
                .battlefield
                .iter()
                .position(|c| c.id == card_id)
                .ok_or(GameError::CardNotOnBattlefield(card_id))?;
            // CR 113.10b — a permanent with all abilities stripped (Turn to
            // Frog / Mercurial Transformation) can't have its printed
            // activated abilities used. We allow mana abilities through (no
            // catalog card stripping abilities has a mana ability of
            // interest right now) by detecting them via `is_mana_ability`.
            // `StaticEffect::GrantActivatedAbility` (Galazeth Prismari,
            // Cryptolith Rite, …): surface granted abilities as virtual
            // activated abilities at indices ≥ printed_count, so standard
            // activate_ability validation and mana payment work without
            // modifying every ability lookup path. Stripped permanents
            // (CR 113.10b) keep their granted mana abilities but lose
            // non-mana grants.
            //
            // Every layer read this activation needs, in one scope — this is
            // a `&mut self` path, so without it each one re-gathers every
            // continuous effect in the game. `creature_source` is the
            // caller's "did a creature make this mana" flag (CR 106.12 /
            // Cursed Totem-style restrictions); it comes off the same
            // `ComputedPermanent` as `stripped`, and no mana has moved yet.
            //
            // The grant/intrinsic halves are indexed only at
            // `ability_index >= printed_count`, and the printed count is a
            // definition read with no layer in it — so decide it first and
            // skip both for a printed index. That is nearly every
            // activation (a land tapping for mana is index 0), and each one
            // was a whole-board `grant_scan` plus a second
            // `computed_permanent`.
            let printed_count = self.battlefield[pos].definition.activated_abilities.len();
            let want_extra = ability_index >= printed_count;
            let (stripped, is_creature, granted, intrinsic, land_mana_lost, cp) =
                self.with_frozen_layers(|g| {
                    let cp = g.computed_permanent(card_id);
                    // CR 305.6 / 612 — the same computed view answers whether a
                    // basic's printed mana ability survived its type line, so
                    // the check below reads it here instead of taking its own
                    // gather (`printed_land_mana_ability_lost` did, 1.28 % of
                    // the simulator). Cheap when the index isn't a printed
                    // single-colour mana ability: `_with` bails on the printed
                    // shape.
                    let land_mana_lost = match (&cp, g.battlefield_find(card_id)) {
                        (Some(c), Some(card)) => Self::printed_land_mana_ability_lost_with(
                            card,
                            ability_index,
                            &c.subtypes.land_types,
                        ),
                        _ => false,
                    };
                    (
                        cp.as_ref().map(|c| c.lost_all_abilities).unwrap_or(false),
                        cp.as_ref().is_some_and(|c| {
                            c.card_types.contains(&crate::card::CardType::Creature)
                        }),
                        if want_extra { g.granted_abilities_for(card_id) } else { Vec::new() },
                        if want_extra { g.intrinsic_land_mana_abilities(card_id) } else { Vec::new() },
                        land_mana_lost,
                        cp,
                    )
                });
            bf_cp = cp;
            if is_creature {
                *creature_mana_before = Some(self.players[p].mana_pool.clone());
            }
            if ability_index < printed_count {
                let raw = self.battlefield[pos]
                    .definition
                    .activated_abilities[ability_index]
                    .clone();
                if stripped && !is_mana_ability(&raw.effect) {
                    return Err(GameError::AbilityIndexOutOfBounds);
                }
                // CR 305.6 / 612 — a basic's intrinsic mana ability follows its
                // *computed* type line, so a rewritten type takes it away.
                if land_mana_lost {
                    return Err(GameError::AbilityIndexOutOfBounds);
                }
                raw
            } else if ability_index < printed_count + granted.len() {
                let g = granted[ability_index - printed_count].clone();
                // Stripped permanents keep granted mana abilities only.
                if stripped && !is_mana_ability(&g.effect) {
                    return Err(GameError::AbilityIndexOutOfBounds);
                }
                g
            } else if ability_index < printed_count + granted.len() + intrinsic.len() {
                // Intrinsic basic-land mana abilities survive stripping.
                intrinsic[ability_index - printed_count - granted.len()].clone()
            } else {
                return Err(GameError::AbilityIndexOutOfBounds);
            }
        };

        // {X} activation costs ({X}, {T}: … — Berta, Imbraham): a
        // hand-paying activator who didn't send an X picks one via a
        // `ChooseAmount` modal (suspend + clean replay — nothing has been
        // paid yet). The auto path keeps `x_value` as-is (unwrapped to 0
        // downstream, the historical behavior). [`manual_mana`] rather than
        // `wants_ui` for the same reason as the cast_spell X prompt.
        if ability.mana_cost.has_x()
            && x_value.is_none()
            && self.players[p].manual_mana
        {
            let max = self.max_prompt_x(p, &ability.mana_cost);
            let source_name = self
                .find_card_anywhere(card_id)
                .map(|c| c.definition.name.to_string())
                .unwrap_or_default();
            self.pending_decision = Some(crate::game::types::PendingDecision {
                decision: crate::decision::Decision::ChooseAmount {
                    source: card_id,
                    max,
                    prompt: format!("{source_name}: choose X"),
                },
                resume: crate::game::types::ResumeContext::ActivateAbilityChoice {
                    activator: p,
                    card_id,
                    ability_index,
                    target,
                    additional_targets,
                    x_value: None,
                    kind: crate::game::types::AbilityCostChoice::XValue,
                },
            });
            return Ok(vec![]);
        }

        // Spend context for restricted mana: an ArtifactOnly pool entry
        // (Power Depot) may fund abilities of artifact sources.
        let ability_spend_kind = {
            let def = if source_in_gy {
                self.players[source_owner.unwrap()].graveyard.iter()
                    .find(|c| c.id == card_id).map(|c| &c.definition)
            } else if source_in_hand {
                self.players[source_owner.unwrap()].hand.iter()
                    .find(|c| c.id == card_id).map(|c| &c.definition)
            } else if source_in_exile {
                self.exile.iter().find(|c| c.id == card_id).map(|c| &c.definition)
            } else if source_in_command {
                self.players[source_owner.unwrap()].command.iter()
                    .find(|c| c.id == card_id).map(|c| &c.definition)
            } else {
                self.battlefield.iter().find(|c| c.id == card_id).map(|c| &c.definition)
            };
            def.map(|d| d.ability_spend_kind()).unwrap_or_default()
        };

        // For graveyard/hand activations, reject if the ability isn't flagged
        // for that zone. This prevents activating a card's printed
        // battlefield-only ability from another zone accidentally.
        if source_in_gy && !ability.from_graveyard {
            return Err(GameError::CardNotOnBattlefield(card_id));
        }
        if source_in_hand && !ability.from_hand {
            return Err(GameError::CardNotOnBattlefield(card_id));
        }
        if source_in_exile && !ability.from_exile {
            return Err(GameError::CardNotOnBattlefield(card_id));
        }
        if source_in_command && !ability.from_command_zone {
            return Err(GameError::CardNotOnBattlefield(card_id));
        }

        // Only the controller (or graveyard/hand owner) can activate abilities,
        // except abilities flagged `opponents_only` (CR 602.5 — Detention
        // Vortex's escape clause), which only an opponent of the controller may.
        if source_in_gy || source_in_hand || source_in_exile || source_in_command {
            if source_owner != Some(p) {
                return Err(GameError::NotYourPriority);
            }
        } else {
            let controller = self
                .battlefield
                .iter()
                .find(|c| c.id == card_id)
                .ok_or(GameError::CardNotOnBattlefield(card_id))?
                .controller;
            if ability.opponents_only {
                if self.same_team(controller, p) {
                    return Err(GameError::NotYourPriority);
                }
            } else if !ability.any_player && controller != p {
                return Err(GameError::NotYourPriority);
            }
        }

        // CR 201.3 — Pithing Needle / Phyrexian Revoker: a permanent that
        // named this source's card name shuts off its activated abilities
        // unless they're mana abilities. The suppression is global (affects
        // every player's matching sources), so we scan the whole battlefield
        // for a `named_card` matching this source's printed name.
        if !is_mana_ability(&ability.effect) {
            let source_name = if source_in_gy {
                self.players[source_owner.unwrap()].graveyard.iter()
                    .find(|c| c.id == card_id)
                    .map(|c| c.definition.name)
            } else {
                self.battlefield.iter().find(|c| c.id == card_id).map(|c| c.definition.name)
            };
            if let Some(name) = source_name
                && self.battlefield.iter().any(|c| c.named_card.as_deref() == Some(name))
            {
                return Err(GameError::AbilitySuppressedByNamedCard);
            }
        }

        // Hand to Hand — no non-mana activations during combat.
        if !is_mana_ability(&ability.effect) && self.combat_spell_lock_active() {
            return Err(GameError::AbilitySuppressedByNamedCard);
        }

        // Interdict — "that permanent's activated abilities can't be
        // activated this turn". Mana abilities are unaffected.
        if !is_mana_ability(&ability.effect) && self.abilities_locked_this_turn.contains(&card_id) {
            return Err(GameError::AbilitySuppressedByNamedCard);
        }

        // Collector Ouphe / Karn lock: non-mana activated abilities of
        // artifacts can't be activated while a `ArtifactActivatedAbilitiesLocked`
        // static is in play (global — affects every player). A source on the
        // battlefield is checked for its artifact type; gy/hand sources of an
        // artifact (rare) are caught the same way.
        if !is_mana_ability(&ability.effect) {
            let src_is_artifact = if source_in_gy {
                self.players[source_owner.unwrap()].graveyard.iter()
                    .find(|c| c.id == card_id)
                    .is_some_and(|c| c.definition.is_artifact())
            } else if source_in_hand {
                self.players[source_owner.unwrap()].hand.iter()
                    .find(|c| c.id == card_id)
                    .is_some_and(|c| c.definition.is_artifact())
            } else {
                self.battlefield_find(card_id).is_some_and(|c| c.definition.is_artifact())
            };
            if src_is_artifact
                && self.battlefield.iter().flat_map(|c| &c.definition.static_abilities).any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::ArtifactActivatedAbilitiesLocked)
                })
            {
                return Err(GameError::AbilitySuppressedByNamedCard);
            }
        }

        // Karn, the Great Creator — activated abilities (mana abilities
        // included) of artifacts controlled by an opponent of Karn's
        // controller can't be activated.
        {
            let src_artifact_on_bf = !source_in_gy
                && !source_in_hand
                && self.battlefield_find(card_id).is_some_and(|c| c.definition.is_artifact());
            if src_artifact_on_bf
                && self.battlefield.iter().any(|c| {
                    c.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            sa.effect,
                            crate::effect::StaticEffect::OpponentsCantActivateArtifactAbilities
                        )
                    }) && !self.same_team(c.controller, p)
                })
            {
                return Err(GameError::AbilitySuppressedByNamedCard);
            }
        }

        // Grand Abolisher — during the active player's turn, that player's
        // opponents can't activate abilities of artifacts, creatures, or
        // enchantments (mana abilities included). Lands and other permanents
        // are unaffected.
        {
            let active = self.active_player_idx;
            let src_is_ace = !source_in_gy
                && !source_in_hand
                && self.battlefield_find(card_id).is_some_and(|c| {
                    c.definition.is_artifact()
                        || c.definition.card_types.contains(&crate::card::CardType::Creature)
                        || c.definition.card_types.contains(&crate::card::CardType::Enchantment)
                });
            if src_is_ace
                && p != active
                && !self.same_team(p, active)
                && self.battlefield.iter().any(|c| {
                    c.controller == active
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(
                                sa.effect,
                                crate::effect::StaticEffect::OpponentsCantActDuringYourTurn
                            )
                        })
                })
            {
                return Err(GameError::AbilitySuppressedByNamedCard);
            }
        }

        // CR 701.35 — a detained permanent's activated abilities can't be
        // activated (all of them, mana abilities included) until the detainer's
        // next turn.
        if !source_in_gy
            && !source_in_hand
            && self.battlefield_find(card_id).is_some_and(|c| c.detained_by.is_some())
        {
            return Err(GameError::AbilitySuppressedByNamedCard);
        }

        // Cursed Totem / Damping Matrix lock: non-mana activated abilities of
        // creatures can't be activated while a
        // `CreatureActivatedAbilitiesLocked` static is in play (global).
        if !is_mana_ability(&ability.effect) {
            let src_is_creature = if source_in_gy {
                self.players[source_owner.unwrap()].graveyard.iter()
                    .find(|c| c.id == card_id)
                    .is_some_and(|c| c.definition.is_creature())
            } else if source_in_hand {
                self.players[source_owner.unwrap()].hand.iter()
                    .find(|c| c.id == card_id)
                    .is_some_and(|c| c.definition.is_creature())
            } else {
                self.battlefield_find(card_id).is_some_and(|c| c.definition.is_creature())
            };
            if src_is_creature
                && self.battlefield.iter().flat_map(|c| &c.definition.static_abilities).any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::CreatureActivatedAbilitiesLocked)
                })
            {
                return Err(GameError::AbilitySuppressedByNamedCard);
            }
        }

        // The three CR 602.5 gates below all read one bit of *this* card's
        // computed view, nothing between them touches a layer input, and this
        // is a `&mut self` path — so each `computed_permanent` was its own
        // whole-game gather. Take one, under the same conditions that decide
        // whether any of them can fire. On the battlefield that one is
        // `bf_cp`, already taken by the ability lookup above; each gate
        // re-checks its own condition, so handing it a view the old gate
        // would have skipped cannot change an answer.
        let tap_gated = ability.tap_cost || ability.untap_self_cost;
        let on_battlefield = !source_in_gy && !source_in_hand && !source_in_command;
        let cp = if on_battlefield {
            bf_cp
        } else {
            tap_gated.then(|| self.computed_permanent(card_id)).flatten()
        };

        // CR 602.5 — "activated abilities with {T} in their costs can't be
        // activated" (Serra Bestiary). Read off the computed keyword set so a
        // granted restriction applies immediately.
        if tap_gated
            && cp.as_ref().is_some_and(|cp| {
                cp.keywords.contains(&Keyword::CantActivateTapAbilities)
            })
        {
            return Err(GameError::AbilityAlreadyUsedThisTurn);
        }

        // CR 602.5g/h — a creature's ability with a {T} or {Q} cost can't be
        // activated while the creature is summoning-sick, unless it has haste
        // or its controller has a Tyvar-style "as though they had haste" static.
        if tap_gated && on_battlefield {
            let sick = self.battlefield_find(card_id).is_some_and(|c| {
                c.summoning_sick
                    && cp.as_ref().is_some_and(|cp| {
                        cp.card_types.contains(&crate::card::CardType::Creature)
                            && !cp.keywords.contains(&Keyword::Haste)
                    })
            });
            if sick {
                let exempt = self.battlefield.iter().any(|c| {
                    c.controller == p
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(
                                sa.effect,
                                crate::effect::StaticEffect::ControllerCreatureAbilitiesAsThoughHaste
                            )
                        })
                });
                if !exempt {
                    return Err(GameError::SummoningSickness(card_id));
                }
            }
        }

        // CR 602.5c — a permanent whose *computed* keyword set carries
        // `CantActivateAbilities` (Detention Vortex's Aura grant, etc.) can't
        // activate its non-mana abilities. Battlefield sources only.
        if on_battlefield
            && !is_mana_ability(&ability.effect)
            && cp.as_ref().is_some_and(|c| c.keywords.contains(&Keyword::CantActivateAbilities))
        {
            return Err(GameError::AbilitySuppressedByNamedCard);
        }

        // Once-per-turn: reject if this ability index has already been
        // used since the most recent turn-cleanup. The ability is recorded
        // as "used" *after* successful activation below so failed mana
        // payments / illegal targets don't burn the per-turn budget.
        // (Graveyard activations don't track per-card once-per-turn state
        // since the card may move between zones; the gate is no-op.)
        // CR 602.5f — "activate only once each turn" and its "no more than N
        // times each turn" generalization share the permanent's tally.
        let per_turn_cap = ability
            .max_activations_per_turn
            .or(ability.once_per_turn.then_some(1));
        if !source_in_gy
            && !source_in_hand
            && !source_in_exile
            && !source_in_command
            && let Some(cap) = per_turn_cap
        {
            let perm = self
                .battlefield
                .iter()
                .find(|c| c.id == card_id)
                .ok_or(GameError::CardNotOnBattlefield(card_id))?;
            let used = perm.once_per_turn_used.iter().filter(|i| **i == ability_index).count();
            if used as u32 >= cap {
                return Err(GameError::AbilityAlreadyUsedThisTurn);
            }
        }
        // CR 702.56 — Forecast / other hand-activated "once each turn"
        // abilities (the card stays in hand). The hand instance's
        // per-turn budget rides the global `triggered_once_per_turn_used`
        // set, which is cleared at turn cleanup.
        if (source_in_hand || source_in_command)
            && ability.once_per_turn
            && self.triggered_once_per_turn_used.contains(&(card_id, ability_index))
        {
            return Err(GameError::AbilityAlreadyUsedThisTurn);
        }

        // CR 702.177 — Exhaust: an exhaust ability can be activated only once
        // per game. `exhausted_abilities` (never cleared at turn start) records
        // spent indices on the source permanent.
        if !source_in_gy && !source_in_hand && !source_in_exile && !source_in_command && (ability.exhaust || ability.activate_once) {
            let perm = self
                .battlefield
                .iter()
                .find(|c| c.id == card_id)
                .ok_or(GameError::CardNotOnBattlefield(card_id))?;
            if perm.exhausted_abilities.contains(&ability_index) {
                return Err(GameError::AbilityAlreadyUsedThisTurn);
            }
        }

        // Sorcery-speed gate: reject the activation if the ability is
        // flagged sorcery-speed and the controller can't currently
        // cast sorceries (not their main phase, or stack non-empty).
        // Used by cards with printed "Activate only as a sorcery" — e.g.
        // SOS Summoned Dromedary, Stone Docent, Cauldron of Essence's
        // reanimation. The pre-fix flow let upkeep activations leak
        // through silently.
        if ability.sorcery_speed && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }

        // Per-card "activate only if …" gate. Evaluated against the
        // controller/source context before any cost is paid. Used by
        // cards like Resonating Lute (`{T}: Draw a card. Activate only
        // if you have seven or more cards in your hand.`), Potioner's
        // Trove (`{T}: gain 2 life. Activate only if you've cast an IS
        // spell this turn.`), and similar conditional activations.
        if let Some(cond) = &ability.condition {
            let ctx = crate::game::effects::EffectContext {
                controller: p,
                source: Some(card_id),
                targets: vec![],
                trigger_source: None,
                mode: 0,
                // The gate can read the announced X (Soul Foundry's "X is the
                // mana value of that card").
                x_value: x_value.unwrap_or(0),
                converged_value: 0,
                mana_spent: 0,
                mana_spent_by_color: Vec::new(),
                source_name: None,
                cast_from_hand: true,
                event_amount: 0,
                kicked: false,
                kicked_options: Vec::new(),
                kick_count: 0,
                bargained: false,
                cast_via_mayhem: false,
                cast_via_waterbend: false,
                cast_collected_evidence: false,
                    entwined: false,
                    spree_modes: Vec::new(),
            };
            if !self.evaluate_predicate(cond, &ctx) {
                return Err(GameError::AbilityConditionNotMet);
            }
        }

        // Reject the activation if the chosen target has hexproof / shroud /
        // protection / Leyline-of-Sanctity-style player hexproof. Mana-only
        // and self-targeting abilities don't pass a target so they bypass.
        if let Some(tgt) = &target {
            self.check_target_legality(tgt, p)?;
            if self.ability_target_has_protection(tgt, card_id) {
                return Err(GameError::TargetHasProtection(card_id));
            }
            // Ward enforcement happens via push_ward_triggers_for_cast
            // after finalize_cast, not as a synchronous cost payment.
            let _ = tgt; let _ = p;
        }

        // Enforce the ability's own target selection requirement (e.g.
        // Wasteland's "destroy target nonbasic land", Goblin Bombardment's
        // "any target"). Spell casts already validate this in `cast_spell`;
        // activated abilities went unchecked, which let bots/UIs aim a
        // Wasteland at a Plains. Mirror the cast-side gate for parity.
        if let Some(tgt) = &target
            && let Some(filter) = ability
                .effect
                .target_filter_for_slot_in_mode(0, chosen_mode)
                .map(|f| f.resolve_x(x_value.unwrap_or(0)))
            && !self.evaluate_requirement_static(&filter, tgt, p, Some(card_id))
        {
            return Err(GameError::SelectionRequirementViolated);
        }

        // Two-target activated abilities (Autumn-Tail): validate slots 1+ the
        // same way — legality (hexproof/shroud/…) plus the per-slot filter.
        for (i, tgt) in additional_targets.iter().enumerate() {
            self.check_target_legality(tgt, p)?;
            if self.ability_target_has_protection(tgt, card_id) {
                return Err(GameError::TargetHasProtection(card_id));
            }
            if let Some(filter) = ability
                .effect
                .target_filter_for_slot((i + 1) as u8)
                .map(|f| f.resolve_x(x_value.unwrap_or(0)))
                && !self.evaluate_requirement_static(&filter, tgt, p, Some(card_id))
            {
                return Err(GameError::SelectionRequirementViolated);
            }
        }
        // CR 601.2c — Flagbearer applies to activated abilities too.
        {
            let chosen: Vec<Target> =
                target.iter().cloned().chain(additional_targets.iter().cloned()).collect();
            let slots: Vec<Option<crate::card::SelectionRequirement>> = (0..chosen.len())
                .map(|i| ability.effect.target_filter_for_slot_in_mode(i as u8, chosen_mode).cloned())
                .collect();
            if self.flagbearer_violation(p, &chosen, &slots) {
                return Err(GameError::InvalidTarget);
            }
        }
        // Reject when the effect still references a target slot we weren't
        // given (a two-target ability invoked with too few targets — the
        // bot / affordance path passes only slot 0). Without this the extra
        // slot resolves to nothing and the effect half-fires.
        // A divided-damage / "up to N targets" ability's slots past its minimum
        // are optional (CR 115.3), so only a genuinely required slot rejects.
        let next_slot = 1 + additional_targets.len() as u8;
        if target.is_some()
            && ability.effect.target_filter_for_slot_in_mode(next_slot, chosen_mode).is_some()
            && !ability.effect.target_slot_optional_x(
                next_slot,
                chosen_mode,
                x_value.unwrap_or(0),
            )
        {
            return Err(GameError::SelectionRequirementViolated);
        }

        // Graveyard-card targets (reanimation abilities — Cauldron of
        // Essence's "Return target creature card from your graveyard"):
        // the in-scene cursor can't select graveyard cards, so clients
        // activate with `target: None`. Bind slot 0 here — a `wants_ui`
        // activator with a real choice picks via a `ChooseCards` modal
        // (suspend + clean replay, like the cost picks below); otherwise
        // auto-pick. Without this the effect went on the stack unbound
        // and silently no-opped.
        let mut target = target;
        if target.is_none()
            && ability.effect.prefers_graveyard_target()
            && let Some(filter) = ability
                .effect
                .target_filter_for_slot(0)
                .map(|f| f.resolve_x(x_value.unwrap_or(0)))
        {
            let candidates: Vec<CardId> = self
                .players
                .iter()
                .flat_map(|pl| pl.graveyard.iter())
                .filter(|c| c.id != card_id)
                .filter(|c| {
                    self.evaluate_requirement_static(
                        &filter,
                        &Target::Permanent(c.id),
                        p,
                        Some(card_id),
                    )
                })
                .map(|c| c.id)
                .collect();
            if candidates.is_empty() {
                return Err(GameError::SelectionRequirementViolated);
            }
            if candidates.len() > 1 && self.players[p].wants_ui {
                let source_name = self
                    .find_card_anywhere(card_id)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                let named: Vec<(CardId, String)> = candidates
                    .iter()
                    .map(|id| {
                        (
                            *id,
                            self.find_card_anywhere(*id)
                                .map(|c| c.definition.name.to_string())
                                .unwrap_or_default(),
                        )
                    })
                    .collect();
                self.pending_decision = Some(crate::game::types::PendingDecision {
                    decision: crate::decision::Decision::ChooseCards {
                        source: card_id,
                        prompt: format!(
                            "{source_name}: choose a card in a graveyard to target"
                        ),
                        candidates: named,
                        min: 1,
                        max: 1,
                    },
                    resume: crate::game::types::ResumeContext::ActivateAbilityChoice {
                        activator: p,
                        card_id,
                        ability_index,
                        target: None,
                        additional_targets: additional_targets.clone(),
                        x_value,
                        kind: crate::game::types::AbilityCostChoice::GraveyardTarget,
                    },
                });
                return Ok(vec![]);
            }
            // Auto-pick the highest-MV candidate (reanimation-style effects
            // want the biggest card back).
            target = candidates
                .iter()
                .copied()
                .max_by_key(|id| {
                    self.find_card_anywhere(*id)
                        .map(|c| c.definition.cost.cmc())
                        .unwrap_or(0)
                })
                .map(Target::Permanent);
        }

        // Pre-flight life-cost gate: reject activation cleanly when the
        // controller doesn't have enough life. Mirror the mana-cost
        // pre-pay check (we want a clean error, not a "you can't pay
        // and just lost a tap" surprise). Activation that gets past
        // this point will deduct the life after tap/mana succeed.
        if ability.life_cost > 0 && self.players[p].life < ability.life_cost as i32 {
            return Err(GameError::InsufficientLife);
        }
        // Pre-flight variable life-cost gate ("Pay X life", CR 107.16): the
        // spend equals the activation's chosen `x_value`. Reject cleanly when
        // short so tap/mana aren't burned (Krumar Initiate).
        if ability.x_life_cost && self.players[p].life < x_value.unwrap_or(0) as i32 {
            return Err(GameError::InsufficientLife);
        }
        // "Pay half your life, rounded up" (CR 118.4 — Lurking Evil).
        if ability.half_life_cost && self.players[p].life <= 0 {
            return Err(GameError::InsufficientLife);
        }

        // Pre-flight {E} gate (CR 107.16): reject cleanly when the controller
        // lacks the energy. Mirrors the mana/life pre-pay checks; the spend
        // happens after tap/mana/life succeed.
        if ability.energy_cost > 0 && self.players[p].energy < ability.energy_cost {
            return Err(GameError::InsufficientEnergy);
        }
        // Pre-flight variable-{E} gate: "Pay X {E}" abilities spend the
        // activation's chosen `x_value` energy (CR 107.16); reject cleanly when
        // short. The same X gates the target filter (ManaValueExactlyXFromCost).
        if ability.energy_x_cost && self.players[p].energy < x_value.unwrap_or(0) {
            return Err(GameError::InsufficientEnergy);
        }

        // Pre-flight collect-evidence gate (CR 701.59): reject cleanly when the
        // graveyard can't supply the required total mana value, so tap/mana
        // aren't burned. The exile happens after payment succeeds.
        if let Some(amount) = ability.collect_evidence_cost
            && !self.graveyard_can_collect_evidence(p, amount)
        {
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pre-flight exile-other-from-gy gate: confirm `count` graveyard
        // cards matching the cost's filter exist, *excluding* the source
        // itself for graveyard activations where source_in_gy is true. If
        // fewer than `count` match, reject cleanly so tap/mana aren't burned.
        // The actual exile happens after payment succeeds.
        //
        // CR 602.5b — a `wants_ui` activator with more candidates than required
        // chooses which cards to exile via a `ChooseCards` modal (graveyard
        // cards aren't selectable with the in-scene cursor). Bots and the
        // no-real-choice case keep the lowest-CMC auto-pick so higher-value
        // graveyard cards stay put. Affects Grim Lavamancer, Scrapheap
        // Scrounger, et al.
        // Exile-a-card-from-HAND as an additional cost (Holistic Wisdom).
        // Validated here so tap/mana aren't burned on an unpayable cost; the
        // exile itself happens once payment succeeds.
        let exile_from_hand_pick: Option<CardId> =
            if let Some(filter) = ability.exile_from_hand_cost.as_ref() {
                let pick = self.players[p]
                    .hand
                    .iter()
                    .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                    .min_by_key(|c| c.definition.cost.cmc())
                    .map(|c| c.id);
                match pick {
                    None => return Err(GameError::SelectionRequirementViolated),
                    some => some,
                }
            } else {
                None
            };

        let exile_other_picks: Vec<CardId> = if let Some((filter, count)) =
            ability.exile_other_filter.as_ref()
        {
            // "Exile X cards from your graveyard:" costs read the activation's X.
            let count = if ability.exile_other_x {
                x_value.unwrap_or(0) as usize
            } else {
                *count as usize
            };
            let candidates: Vec<CardId> = self.players[p]
                .graveyard
                .iter()
                .filter(|c| c.id != card_id)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| c.id)
                .collect();
            if candidates.len() < count {
                return Err(GameError::SelectionRequirementViolated);
            }
            if ability.exile_other_top {
                // "Exile the top [filter] card of your graveyard" — the
                // graveyard is ordered, so the last matches are forced.
                candidates.iter().rev().copied().take(count).collect()
            } else if let Some(chosen) = chosen_exile_other {
                // Replay path: keep the player's picks that are still valid
                // candidates; backfill from the auto-pick if short.
                let valid: std::collections::HashSet<CardId> = candidates.iter().copied().collect();
                let mut picks: Vec<CardId> =
                    chosen.into_iter().filter(|id| valid.contains(id)).take(count).collect();
                if picks.len() < count {
                    let have: std::collections::HashSet<CardId> = picks.iter().copied().collect();
                    let extra: Vec<CardId> = candidates.iter().copied().filter(|id| !have.contains(id)).collect();
                    picks.extend(self.auto_pick_lowest_cmc_gy(p, &extra, count - picks.len()));
                }
                picks
            } else if candidates.len() > count && self.players[p].manual_mana {
                let source_name = self
                    .battlefield_find(card_id)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                let named = self.graveyard_card_names(p, &candidates);
                self.pending_decision = Some(crate::game::types::PendingDecision {
                    decision: crate::decision::Decision::ChooseCards {
                        source: card_id,
                        prompt: format!("{source_name}: exile {count} cards from your graveyard"),
                        candidates: named,
                        min: count as u32,
                        max: count as u32,
                    },
                    resume: crate::game::types::ResumeContext::ActivateAbilityChoice {
                        activator: p,
                        card_id,
                        ability_index,
                        target,
                        additional_targets: additional_targets.clone(),
                        x_value,
                        kind: crate::game::types::AbilityCostChoice::ExileOther,
                    },
                });
                return Ok(vec![]);
            } else {
                self.auto_pick_lowest_cmc_gy(p, &candidates, count)
            }
        } else {
            Vec::new()
        };

        // Pre-flight Craft gate (CR 702.169): collect `count` *other* objects
        // matching the filter from among permanents the activator controls
        // and/or cards in their graveyard. Reject cleanly when fewer than
        // `count` match. Auto-picks graveyard cards first, then the lowest-power
        // battlefield permanents, so higher-value board pieces stay put. The
        // exile happens after tap/mana/life payment succeeds. (`(card_id, true)`
        // marks a graveyard pick; `(id, false)` a battlefield pick.)
        let craft_exile_picks: Vec<(CardId, bool)> = if let Some((filter, count)) =
            ability.craft_exile_cost.as_ref()
        {
            let count = *count as usize;
            let gy: Vec<CardId> = self.players[p]
                .graveyard
                .iter()
                .filter(|c| c.id != card_id)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| c.id)
                .collect();
            let mut bf: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.id != card_id && c.controller == p)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| c.id)
                .collect();
            if gy.len() + bf.len() < count {
                return Err(GameError::SelectionRequirementViolated);
            }
            let mut picks: Vec<(CardId, bool)> =
                gy.into_iter().take(count).map(|id| (id, true)).collect();
            if picks.len() < count {
                let need = count - picks.len();
                bf = self.auto_pick_lowest_power(&bf, need);
                picks.extend(bf.into_iter().map(|id| (id, false)));
            }
            picks
        } else {
            Vec::new()
        };

        // Pre-flight "Exile a [filter] you control:" gate (Food Chain). Rejects
        // cleanly when too few match; the auto-picker takes the lowest-power
        // candidates so better creatures stay on the battlefield.
        let exile_permanent_picks: Vec<CardId> = if let Some((filter, count)) =
            ability.exile_permanent_cost.as_ref()
        {
            let candidates: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| c.id)
                .collect();
            if candidates.len() < *count as usize {
                return Err(GameError::SelectionRequirementViolated);
            }
            self.auto_pick_lowest_power(&candidates, *count as usize)
        } else {
            Vec::new()
        };

        // Pre-flight sacrifice-other gate: confirm `count` battlefield
        // permanents the activator controls match the cost's filter
        // (excluding the source itself, since activating from the
        // battlefield can pair this with `sac_cost: true` for the source).
        // If fewer than `count` match, reject cleanly so tap/mana aren't burned.
        //
        // CR 602.5b — the activator chooses which permanent(s) to sacrifice as
        // a cost. For a `wants_ui` activator making a *single* such sacrifice
        // with a genuine choice (more than one legal candidate) we suspend on a
        // `ChooseTarget` and replay the activation with the pick. Bots,
        // multi-sacrifice (count > 1), and "no real choice" keep the
        // lowest-power auto-pick (so the activator keeps better creatures).
        let sac_other_picks: Vec<CardId> = if let Some((filter, count)) =
            ability.sac_other_filter.as_ref()
        {
            // "Sacrifice X [filter]:" costs read the activation's X.
            let count = if ability.sac_other_x {
                x_value.unwrap_or(0) as usize
            } else {
                *count as usize
            };
            // `AttachedToSource` in the cost filter means "attached to *this*
            // permanent" — a source-precise check the source-blind
            // `evaluate_requirement_on_card` can't make, so intersect the
            // source id here (Faunsbane Troll — "Sacrifice an Aura attached to
            // this creature").
            let needs_attached_to_source =
                crate::game::requirement_mentions_attached_to_source(filter);
            let needs_host_of_source = crate::game::requirement_mentions_host_of_source(filter);
            let host = self.battlefield_find(card_id).and_then(|c| c.attached_to);
            let candidates: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.id != card_id && c.controller == p)
                .filter(|c| !needs_attached_to_source || c.attached_to == Some(card_id))
                .filter(|c| !needs_host_of_source || host == Some(c.id))
                .map(|c| c.id)
                .collect::<Vec<_>>()
                .into_iter()
                // Battlefield-aware: the cost filter reads computed types and
                // live counters (Ambush Commander's animated Forests, Trap
                // Digger's "land with a trap counter on it").
                .filter(|id| {
                    self.evaluate_requirement_static(
                        filter,
                        &Target::Permanent(*id),
                        p,
                        Some(card_id),
                    )
                })
                .collect();
            if candidates.len() < count {
                return Err(GameError::SelectionRequirementViolated);
            }
            if let Some(chosen) = chosen_sac_other {
                // Replay path: honor the player's pick if it's still a valid
                // candidate; otherwise fall back to the auto-pick.
                if candidates.contains(&chosen) {
                    vec![chosen]
                } else {
                    self.auto_pick_lowest_power(&candidates, count)
                }
            } else if count == 1 && candidates.len() > 1 && self.players[p].manual_mana {
                let source_name = self
                    .battlefield_find(card_id)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                self.pending_decision = Some(crate::game::types::PendingDecision {
                    decision: crate::decision::Decision::ChooseTarget {
                        optional: false,
                        source: card_id,
                        legal: candidates.iter().map(|id| Target::Permanent(*id)).collect(),
                        source_name,
                        description: "choose a permanent to sacrifice (cost)".into(),
                    },
                    resume: crate::game::types::ResumeContext::ActivateAbilityChoice {
                        activator: p,
                        card_id,
                        ability_index,
                        target,
                        additional_targets: additional_targets.clone(),
                        x_value,
                        kind: crate::game::types::AbilityCostChoice::SacOther,
                    },
                });
                return Ok(vec![]);
            } else {
                self.auto_pick_lowest_power(&candidates, count)
            }
        } else {
            Vec::new()
        };

        let mut sac_other_picks = sac_other_picks;

        // CR 602.5b — "…and any number of [filter] you control". Zero is a
        // legal payment, so there is no candidate-count gate; a hand-paying
        // activator picks the subset, everything else sacrifices all of them
        // (Sword of the Ages is a finisher — a partial auto-pick has no
        // sensible size).
        if let Some(filter) = ability.sac_any_number_filter.as_ref() {
            let candidates: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.id != card_id && c.controller == p)
                .map(|c| c.id)
                .collect::<Vec<_>>()
                .into_iter()
                .filter(|id| {
                    self.evaluate_requirement_static(filter, &Target::Permanent(*id), p, Some(card_id))
                })
                .collect();
            match chosen_sac_any {
                Some(chosen) => {
                    sac_other_picks.extend(chosen.into_iter().filter(|id| candidates.contains(id)));
                }
                None if !candidates.is_empty() && self.players[p].manual_mana => {
                    let source_name = self
                        .battlefield_find(card_id)
                        .map(|c| c.definition.name.to_string())
                        .unwrap_or_default();
                    let named: Vec<(CardId, String)> = candidates
                        .iter()
                        .filter_map(|id| {
                            self.battlefield_find(*id)
                                .map(|c| (*id, c.definition.name.to_string()))
                        })
                        .collect();
                    let max = named.len() as u32;
                    self.pending_decision = Some(crate::game::types::PendingDecision {
                        decision: crate::decision::Decision::ChooseCards {
                            source: card_id,
                            prompt: format!("{source_name}: sacrifice any number of them"),
                            candidates: named,
                            min: 0,
                            max,
                        },
                        resume: crate::game::types::ResumeContext::ActivateAbilityChoice {
                            activator: p,
                            card_id,
                            ability_index,
                            target,
                            additional_targets: additional_targets.clone(),
                            x_value,
                            kind: crate::game::types::AbilityCostChoice::SacAnyNumber,
                        },
                    });
                    return Ok(vec![]);
                }
                None => sac_other_picks.extend(candidates),
            }
        }

        // CR 602.5b — statics that bolt an extra "Sacrifice a [filter]" onto
        // matching permanents' activated abilities (Brutal Suppression).
        // Mana abilities are exempt, matching the other activation taxes.
        if !is_mana_ability(&ability.effect) {
            let taxes: Vec<crate::card::SelectionRequirement> = self
                .battlefield
                .iter()
                .flat_map(|c| c.definition.static_abilities.iter())
                .filter_map(|sa| match &sa.effect {
                    crate::effect::StaticEffect::ActivationAdditionalSacrifice {
                        filter,
                        sacrifice,
                    } => Some((filter.clone(), sacrifice.clone())),
                    _ => None,
                })
                .filter(|(filter, _)| {
                    self.battlefield_find(card_id)
                        .is_some_and(|c| self.evaluate_requirement_on_card(filter, c, p))
                })
                .map(|(_, sacrifice)| sacrifice)
                .collect();
            for sacrifice in taxes {
                let pick = self
                    .battlefield
                    .iter()
                    .filter(|c| c.controller == p && !sac_other_picks.contains(&c.id))
                    .find(|c| self.evaluate_requirement_on_card(&sacrifice, c, p))
                    .map(|c| c.id);
                match pick {
                    Some(id) => sac_other_picks.push(id),
                    None => return Err(GameError::SelectionRequirementViolated),
                }
            }
        }

        // Pre-flight tap-another gate (CR 602.5b): confirm an untapped
        // permanent (other than the source) the activator controls matches the
        // cost's filter. A hand-paying activator with more than one candidate
        // chooses which to tap (suspend + replay, like the sacrifice cost);
        // bots and the no-real-choice case tap the lowest-power match so
        // higher-value creatures stay open — *unless* the payoff scales with the
        // tapped creature's power (Station's charge add, CR 702.184a), in which
        // case tapping the highest-power creature is strictly better.
        // Tapped after payment succeeds.
        let prefer_highest_tap = serde_json::to_string(&ability.effect)
            .map(|s| s.contains("TappedForCostPower"))
            .unwrap_or(false);
        let auto_tap_pick = |g: &Self, candidates: &[CardId]| -> Option<CardId> {
            if prefer_highest_tap {
                g.auto_pick_highest_power(candidates, 1).first().copied()
            } else {
                g.auto_pick_lowest_power(candidates, 1).first().copied()
            }
        };
        let tap_other_pick: Option<CardId> = if let Some(filter) =
            ability.tap_other_filter.as_ref()
        {
            let host = self.battlefield_find(card_id).and_then(|c| c.attached_to);
            let needs_host_of_source = crate::game::requirement_mentions_host_of_source(filter);
            let candidates: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.id != card_id && c.controller == p && !c.tapped)
                .filter(|c| !needs_host_of_source || host == Some(c.id))
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| c.id)
                .collect();
            if candidates.is_empty() {
                return Err(GameError::SelectionRequirementViolated);
            }
            if let Some(chosen) = chosen_tap_other {
                // Replay path: honor the pick if still a valid candidate.
                if candidates.contains(&chosen) {
                    Some(chosen)
                } else {
                    auto_tap_pick(self, &candidates)
                }
            } else if candidates.len() > 1 && self.players[p].manual_mana {
                let source_name = self
                    .battlefield_find(card_id)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                self.pending_decision = Some(crate::game::types::PendingDecision {
                    decision: crate::decision::Decision::ChooseTarget {
                        optional: false,
                        source: card_id,
                        legal: candidates.iter().map(|id| Target::Permanent(*id)).collect(),
                        source_name,
                        description: "choose a permanent to tap (cost)".into(),
                    },
                    resume: crate::game::types::ResumeContext::ActivateAbilityChoice {
                        activator: p,
                        card_id,
                        ability_index,
                        target,
                        additional_targets: additional_targets.clone(),
                        x_value,
                        kind: crate::game::types::AbilityCostChoice::TapOther,
                    },
                });
                return Ok(vec![]);
            } else {
                auto_tap_pick(self, &candidates)
            }
        } else {
            None
        };

        // Pre-flight tap-N gate (CR 602.5b "Tap N untapped … you control:").
        // Confirm `count` untapped matching permanents the activator controls;
        // the source itself may be one of them (Heritage Druid is an Elf).
        // Auto-pick the lowest-power matches (preferring to keep the source).
        let tap_n_picks: Vec<CardId> = if let Some((filter, count)) =
            ability.tap_n_filter.as_ref()
        {
            let count = *count as usize;
            let mut candidates: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && !c.tapped)
                // With a separate {T} cost the source is already being
                // tapped, so it can't double as one of the N ("{T}, Tap
                // two untapped creatures you control" — Harmonized Trio).
                // Heritage Druid-style abilities without {T} still count
                // the source among its own Elves.
                .filter(|c| !(ability.tap_cost && c.id == card_id))
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| c.id)
                .collect();
            if candidates.len() < count {
                return Err(GameError::SelectionRequirementViolated);
            }
            // Sort the source last so it's tapped only if needed.
            candidates.sort_by_key(|id| (*id == card_id, *id));
            candidates.truncate(count);
            candidates
        } else {
            Vec::new()
        };

        // Pre-flight bounce-another gate (CR 602.5b "Return a [filter] you
        // control to its owner's hand:"). Confirm `count` battlefield
        // permanents (other than the source) the activator controls match the
        // filter; auto-pick the lowest-power match. Quirion Ranger, Wirewood
        // Symbiote. (No interactive picker yet — bots/auto only.)
        let bounce_other_picks: Vec<CardId> = if let Some((filter, count)) =
            ability.bounce_other_filter.as_ref()
        {
            let count = *count as usize;
            let candidates: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.id != card_id && c.controller == p)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| c.id)
                .collect();
            if candidates.len() < count {
                return Err(GameError::SelectionRequirementViolated);
            }
            self.auto_pick_lowest_power(&candidates, count)
        } else {
            Vec::new()
        };

        // Pre-flight "put a card from your hand on top of your library" gate
        // (CR 602.5b — Hidden Retreat). Lowest mana value, like discard costs.
        let library_top_pick: Option<CardId> = if ability.put_hand_on_library_cost {
            match self.players[p]
                .hand
                .iter()
                .filter(|c| c.id != card_id)
                .min_by_key(|c| c.definition.cost.cmc())
            {
                Some(c) => Some(c.id),
                None => return Err(GameError::SelectionRequirementViolated),
            }
        } else {
            None
        };

        // Pre-flight discard-cost gate (CR 602.5b "Discard a [filter] card:").
        // Confirm `count` matching cards in the activator's hand; pick the
        // lowest-CMC matches so higher-value cards stay. Discarded after
        // payment succeeds. Fauna Shaman, Survival of the Fittest.
        let discard_picks: Vec<CardId> = if let Some((filter, count)) =
            ability.discard_cost.as_ref()
        {
            let count = *count as usize;
            if ability.discard_cost_same_name {
                // CR 601 — "Discard N cards with the same name." Find any name
                // in hand with `count`+ matching copies; discard that many.
                let mut by_name: std::collections::HashMap<&str, Vec<CardId>> =
                    std::collections::HashMap::new();
                for c in self.players[p].hand.iter().filter(|c| c.id != card_id) {
                    if self.evaluate_requirement_on_card(filter, c, p) {
                        by_name.entry(c.definition.name).or_default().push(c.id);
                    }
                }
                match by_name.values().find(|ids| ids.len() >= count) {
                    Some(ids) => ids.iter().take(count).copied().collect(),
                    None => return Err(GameError::SelectionRequirementViolated),
                }
            } else {
                // CR 601.2b linked X — "discard a card with mana value X" where
                // the same X gates the target (Kozilek, the Great Distortion).
                let target_mv = if ability.discard_cost_matches_target_mv {
                    match target.as_ref().and_then(|t| match t {
                        Target::Permanent(id) => self.find_card_anywhere(*id),
                        Target::Player(_) => None,
                    }) {
                        Some(c) => Some(c.definition.cost.cmc()),
                        None => return Err(GameError::SelectionRequirementViolated),
                    }
                } else {
                    None
                };
                let mut picks: Vec<(CardId, i32)> = self.players[p]
                    .hand
                    .iter()
                    .filter(|c| c.id != card_id)
                    .filter(|c| target_mv.is_none_or(|mv| c.definition.cost.cmc() == mv))
                    .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                    .map(|c| (c.id, c.definition.cost.cmc() as i32))
                    .collect();
                if picks.len() < count {
                    return Err(GameError::SelectionRequirementViolated);
                }
                if ability.discard_cost_random {
                    use rand::seq::SliceRandom;
                    picks.shuffle(&mut self.rng.draw());
                } else {
                    picks.sort_by_key(|(_, cmc)| *cmc);
                }
                picks.into_iter().take(count).map(|(cid, _)| cid).collect()
            }
        } else {
            Vec::new()
        };

        // Pre-flight process gate: "Put N cards an opponent owns from exile
        // into that player's graveyard:" is a real cost, so too few eligible
        // exile cards means the ability can't be activated (Cryptic Cruiser).
        let process_picks: Vec<CardId> = match ability.process_cost {
            Some(count) => {
                let opponents = self.opponents_of(p);
                let picks: Vec<CardId> = self
                    .exile
                    .iter()
                    .filter(|c| opponents.contains(&c.owner))
                    .map(|c| c.id)
                    .take(count as usize)
                    .collect();
                if picks.len() < count as usize {
                    return Err(GameError::SelectionRequirementViolated);
                }
                picks
            }
            None => Vec::new(),
        };

        // Pre-flight exile-a-spell-you-control gate (CR 602.5b "Exile [a
        // spell] you control:"). Find the top-most matching spell the
        // activator controls on the stack; it leaves without resolving.
        // Nivmagus Elemental. Identified by CardId so it can be located
        // again after tap/mana are paid.
        let exile_spell_pick: Option<CardId> = if let Some(filter) =
            ability.exile_spell_cost.as_ref()
        {
            let pick = self.stack.iter().rev().find_map(|item| match item {
                StackItem::Spell { card, caster, .. }
                    if *caster == p && self.evaluate_requirement_on_card(filter, card, p) =>
                {
                    Some(card.id)
                }
                _ => None,
            });
            match pick {
                Some(id) => Some(id),
                None => return Err(GameError::SelectionRequirementViolated),
            }
        } else {
            None
        };

        // Pre-flight remove-counter-cost gate (CR 602.5b "Remove a [kind]
        // counter from this:"). The source must carry `count` counters of
        // the named kind; removed after payment so the ability can't be
        // over-activated off the stack. Walking Ballista, Triskelion,
        // Hangarback Walker.
        if let Some((kind, count)) = ability.remove_counter_cost.as_ref() {
            let have = self
                .battlefield_find(card_id)
                .map(|c| c.counter_count(*kind))
                .unwrap_or(0);
            if have < *count {
                return Err(GameError::SelectionRequirementViolated);
            }
        }

        // Pre-flight remove-ALL-counters gate: "Remove all [kind] counters
        // from this:" needs at least one to pay with (Essence Bottle).
        if let Some(kind) = ability.remove_all_counters_cost.as_ref()
            && self.battlefield_find(card_id).map(|c| c.counter_count(*kind)).unwrap_or(0) == 0
        {
            return Err(GameError::SelectionRequirementViolated);
        }

        // Pre-flight tap-permanents gate (CR 602.5b "Tap N untapped [filter]
        // you control:" — Lullmage Mentor). The tap is paid below, after mana.
        let tap_cost_picks: Vec<CardId> = match ability.tap_permanents_cost.as_ref() {
            Some((filter, count)) => {
                let mut cands: Vec<&crate::card::CardInstance> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == p
                            && !c.tapped
                            && self.evaluate_requirement_static(
                                filter,
                                &Target::Permanent(c.id),
                                p,
                                Some(card_id),
                            )
                    })
                    .collect();
                if cands.len() < *count as usize {
                    return Err(GameError::SelectionRequirementViolated);
                }
                cands.sort_by_key(|c| (c.definition.is_land(), c.definition.cost.cmc()));
                cands.iter().take(*count as usize).map(|c| c.id).collect()
            }
            None => Vec::new(),
        };

        // Pre-flight "Remove X [kind] counters" gate (Arcbound Javelineer).
        if let Some(kind) = ability.remove_counter_x.as_ref() {
            let have = self
                .battlefield_find(card_id)
                .map(|c| c.counter_count(*kind))
                .unwrap_or(0);
            if have < x_value.unwrap_or(0) {
                return Err(GameError::SelectionRequirementViolated);
            }
        }

        // Pre-flight remove-counters-from-among gate (Hopeful Initiate): the
        // matching permanents you control must together carry `count` counters.
        if let Some((kinds, count, filter)) = counter_drain_cost(&ability) {
            let have: u32 = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p)
                .filter(|c| self.evaluate_requirement_static(&filter, &Target::Permanent(c.id), p, Some(card_id)))
                .map(|c| drainable_counters(c, kinds.as_deref()))
                .sum();
            if have < count {
                return Err(GameError::SelectionRequirementViolated);
            }
        }

        // Pre-flight variable remove-counters-from-among gate (Ooze Flux): the
        // matching permanents must together carry at least X (and X ≥ 1).
        if let Some((kind, filter)) = ability.remove_counter_among_x.as_ref() {
            let want = x_value.unwrap_or(0);
            let have: u32 = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p)
                .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, Some(card_id)))
                .map(|c| c.counter_count(*kind))
                .sum();
            if want == 0 || have < want {
                return Err(GameError::SelectionRequirementViolated);
            }
        }

        // Apply self-counter cost reduction (Strixhaven Book artifacts).
        // Subtracts one generic pip per counter of the specified kind on
        // the source permanent. Clamped at the printed generic total via
        // `ManaCost::reduce_generic`. Only applies when the source is on
        // the battlefield (graveyard-activations skip; the source carries
        // no live counter pool there).
        let mut effective_mana_cost = if ability.mana_cost.has_x() {
            // Bind the X value from the action into the printed cost so
            // the player has to actually pay X generic mana. Used by
            // Pernicious Deed's `{X}, Sacrifice this: …` activation,
            // future Walking Ballista-style `{X}` activations.
            match ability.x_mana_color {
                // CR 601.2g — "spend only [colour] mana on X".
                Some(c) => ability.mana_cost.with_x_value_colored(x_value.unwrap_or(0), c),
                None => ability.mana_cost.with_x_value(x_value.unwrap_or(0)),
            }
        } else {
            ability.mana_cost.clone()
        };
        // A generic cost whose amount the game state defines rather than the
        // activator (Bargaining Table's "X is the number of cards in an
        // opponent's hand"). Folded into the printed generic; the body reads
        // it back via `Value::XFromCost`.
        let state_defined_x = ability.generic_cost_value.as_ref().map(|v| {
            let ctx = crate::game::effects::EffectContext::for_trigger(card_id, p, None, 0);
            self.evaluate_value(v, &ctx).max(0) as u32
        });
        if let Some(n) = state_defined_x {
            effective_mana_cost.add_generic(n);
        }
        let activated_x = if let Some(n) = state_defined_x {
            n
        } else if ability.mana_cost.has_x()
            || ability.sac_other_x
            || ability.exile_other_x
            || ability.remove_counter_x.is_some()
            || ability.remove_counter_among_x.is_some()
            || ability.energy_x_cost
            || ability.x_life_cost
        {
            x_value.unwrap_or(0)
        } else {
            0
        };
        if let Some(kind) = ability.self_counter_cost_reduction
            && !source_in_gy
            && let Some(src) = self.battlefield_find(card_id)
        {
            let count = src.counter_count(kind);
            if count > 0 {
                effective_mana_cost.reduce_generic(count);
            }
        }
        // "Pay {1} for each [kind] counter on this creature" (Skeleton
        // Scavengers) — the surcharge mirror of the reduction above.
        if let Some(kind) = ability.mana_cost_per_self_counter
            && let Some(src) = self.battlefield_find(card_id)
        {
            let count = src.counter_count(kind);
            if count > 0 {
                effective_mana_cost.add_generic(count);
            }
        }
        // "Costs {1} less for each [kind] counter on [filter] you control"
        // (Deepwood Denizen).
        if let Some((kind, filter)) = &ability.cost_reduction_per_counter {
            let count: u32 = self.with_frozen_layers(|g| {
                g.battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == p
                            && g.evaluate_requirement_static(
                                filter,
                                &Target::Permanent(c.id),
                                p,
                                Some(card_id),
                            )
                    })
                    .map(|c| c.counter_count(*kind))
                    .sum()
            });
            if count > 0 {
                effective_mana_cost.reduce_generic(count);
            }
        }
        // "Costs {1} less for each [filter] you control" (channel lands).
        if let Some(filter) = &ability.cost_reduction_per {
            let count = self.with_frozen_layers(|g| {
                g.battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == p
                            && g.evaluate_requirement_static(
                                filter,
                                &Target::Permanent(c.id),
                                p,
                                Some(card_id),
                            )
                    })
                    .count() as u32
            });
            if count > 0 {
                effective_mana_cost.reduce_generic(count);
            }
        }
        // "Costs {X} less to activate, where X is this creature's power"
        // (The Dominion Bracelet, granted to its bearer).
        if ability.cost_reduction_per_equipped_power {
            let power = self
                .computed_permanent(card_id)
                .map(|c| c.power.max(0) as u32)
                .unwrap_or(0);
            if power > 0 {
                effective_mana_cost.reduce_generic(power);
            }
        }
        // "Costs {1} less for each [filter] card in your graveyard"
        // (Battlefield Butcher).
        if let Some(filter) = &ability.cost_reduction_per_graveyard {
            let count = self.players[p]
                .graveyard
                .iter()
                .filter(|c| crate::game::layers::requirement_matches_card(filter, c, p))
                .count() as u32;
            if count > 0 {
                effective_mana_cost.reduce_generic(count);
            }
        }
        // Zirda — non-mana activated abilities cost {N} less (generic only),
        // floored at one mana of the printed cost.
        if !is_mana_ability(&ability.effect) && !effective_mana_cost.symbols.is_empty() {
            let total: u32 = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p)
                .flat_map(|c| c.definition.static_abilities.iter())
                .map(|sa| match sa.effect {
                    crate::effect::StaticEffect::ActivationCostReduction { amount } => amount,
                    _ => 0,
                })
                .sum();
            if total > 0 {
                let max_cut = effective_mana_cost.cmc().saturating_sub(1);
                effective_mana_cost.reduce_generic(total.min(max_cut));
            }
        }
        // Biomancer's Familiar / Training Grounds — activated abilities of
        // creatures you control cost {N} less (generic-only, floored at one
        // mana of the printed cost).
        let source_is_your_creature = self
            .battlefield_find(card_id)
            .is_some_and(|c| c.controller == p && c.definition.is_creature());
        if source_is_your_creature && !effective_mana_cost.symbols.is_empty() {
            let total: u32 = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p)
                .flat_map(|c| c.definition.static_abilities.iter())
                .map(|sa| match sa.effect {
                    crate::effect::StaticEffect::YourCreatureActivatedAbilitiesCostLess { amount } => amount,
                    _ => 0,
                })
                .sum();
            if total > 0 {
                let max_cut = effective_mana_cost.cmc().saturating_sub(1);
                effective_mana_cost.reduce_generic(total.min(max_cut));
            }
        }
        // Power Artifact — the enchanted permanent's activated abilities cost
        // {N} less (generic-only, floored at one mana of the printed cost).
        if !effective_mana_cost.symbols.is_empty() {
            let total: u32 = self
                .battlefield
                .iter()
                .filter(|c| c.attached_to == Some(card_id))
                .flat_map(|c| c.definition.static_abilities.iter())
                .map(|sa| match sa.effect {
                    crate::effect::StaticEffect::AttachedActivatedAbilitiesCostLess { amount } => {
                        amount
                    }
                    _ => 0,
                })
                .sum();
            if total > 0 {
                let max_cut = effective_mana_cost.cmc().saturating_sub(1);
                effective_mana_cost.reduce_generic(total.min(max_cut));
            }
        }
        // Boom Scholar — exhaust abilities of your *other* permanents cost {N}
        // less (generic-only, floored at one mana of the printed cost).
        if ability.exhaust && !effective_mana_cost.symbols.is_empty() {
            let total: u32 = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && c.id != card_id)
                .flat_map(|c| c.definition.static_abilities.iter())
                .map(|sa| match sa.effect {
                    crate::effect::StaticEffect::OtherExhaustActivationCostReduction { amount } => {
                        amount
                    }
                    _ => 0,
                })
                .sum();
            if total > 0 {
                let max_cut = effective_mana_cost.cmc().saturating_sub(1);
                effective_mana_cost.reduce_generic(total.min(max_cut));
            }
        }
        // Skyseer's Chariot — abilities of sources with the named card name
        // cost {N} more. Unlike the taxes below this one covers mana abilities.
        if let Some(name) = self.battlefield_find(card_id).map(|c| c.definition.name) {
            let tax: u32 = self
                .battlefield
                .iter()
                .filter(|c| c.named_card.as_deref() == Some(name))
                .flat_map(|c| c.definition.static_abilities.iter())
                .map(|sa| match sa.effect {
                    crate::effect::StaticEffect::NamedSourcesActivationTax { amount } => amount,
                    _ => 0,
                })
                .sum();
            if tax > 0 {
                effective_mana_cost.symbols.push(crate::mana::ManaSymbol::Generic(tax));
            }
        }

        // Suppression Field — non-mana activated abilities cost {N} more,
        // for every player's activations. Tithe Taker adds the same tax but
        // only to opponents' activations on its controller's turn.
        if !is_mana_ability(&ability.effect) {
            let tax: u32 = self
                .battlefield
                .iter()
                .flat_map(|c| {
                    let opp_your_turn = c.controller != p && c.controller == self.active_player_idx;
                    let attached_here = c.attached_to == Some(card_id);
                    c.definition.static_abilities.iter().map(move |sa| match sa.effect {
                        crate::effect::StaticEffect::ActivationTax { amount } => amount,
                        crate::effect::StaticEffect::AttachedActivationTax { amount }
                            if attached_here =>
                        {
                            amount
                        }
                        crate::effect::StaticEffect::OpponentActivityCostsMoreOnYourTurn { amount }
                            if opp_your_turn =>
                        {
                            amount
                        }
                        _ => 0,
                    })
                })
                .sum();
            if tax > 0 {
                effective_mana_cost.symbols.push(crate::mana::ManaSymbol::Generic(tax));
            }
        }

        // CR 601.2g — float-spend confirmation. Before tapping anything, if the
        // activator has pre-existing floating mana the mana cost could either
        // spend or avoid (untapped sources can cover it), ask first. Nothing is
        // mutated yet at this point (tap-cost applies below), so suspending is a
        // clean replay of the whole activation.
        // CR 601.2g float-spend confirmation is a *mana payment* question,
        // so it keys on `manual_mana` — the flag that exists for exactly this
        // rule — rather than `wants_ui`, which bot seats also set. Prompting
        // a bot here is the same livelock as the {X} and additional-cost
        // modals: the suspend returns `Ok`, so the probe reports the cast as
        // legal, and the failed replay is rolled back with the decision
        // restored. Latent rather than observed only because the bot stopped
        // floating mana when it stopped pre-tapping its board.
        if spend_float.is_none()
            && self.players[p].manual_mana
            && !effective_mana_cost.symbols.is_empty()
            && self.float_spend_is_optional(p, &effective_mana_cost, &ability_spend_kind)
        {
            let float_summary = self.protectable_float(p, &effective_mana_cost).summary();
            let name = self
                .battlefield_find(card_id)
                .map(|c| c.definition.name)
                .unwrap_or("this ability");
            self.pending_decision = Some(crate::game::types::PendingDecision {
                decision: crate::decision::Decision::OptionalTrigger {
                    source: card_id,
                    description: format!(
                        "Spend leftover floating mana ({float_summary}) to activate {name}? (No keeps it and taps lands)"
                    ),
                },
                resume: crate::game::types::ResumeContext::ActionFloatConfirm {
                    actor: p,
                    action: Box::new(GameAction::ActivateAbility {
                        card_id,
                        ability_index,
                        target: target.clone(),
                        additional_targets: Vec::new(),
                        x_value, mode: None,
                    }),
                },
            });
            return Ok(vec![]);
        }

        // Snapshot pristine state before applying tap-cost so a failed mana
        // payment rolls back both the auto-tap of mana sources AND the
        // tap-cost on the source itself.
        let needs_payment = !effective_mana_cost.symbols.is_empty();
        let pre_snapshot = needs_payment.then(|| self.snapshot_payment_state(p));

        // Pay tap cost. Graveyard activations can't tap (the source is not
        // a permanent), so we reject any `tap_cost: true` ability from a
        // graveyard source as a guard against malformed card definitions.
        if ability.tap_cost {
            if source_in_gy || source_in_hand || source_in_exile || source_in_command {
                return Err(GameError::CardIsTapped(card_id));
            }
            let perm = self
                .battlefield
                .iter_mut()
                .find(|c| c.id == card_id)
                .ok_or(GameError::CardNotOnBattlefield(card_id))?;
            if perm.tapped {
                return Err(GameError::CardIsTapped(card_id));
            }
            perm.tapped = true;
        }
        // CR 107.17 — pay a {Q} cost: the source must be tapped; untap it.
        if ability.untap_self_cost {
            if source_in_gy || source_in_hand || source_in_exile {
                return Err(GameError::CardIsTapped(card_id));
            }
            let perm = self
                .battlefield
                .iter_mut()
                .find(|c| c.id == card_id)
                .ok_or(GameError::CardNotOnBattlefield(card_id))?;
            if !perm.tapped {
                return Err(GameError::CardIsTapped(card_id));
            }
            perm.tapped = false;
        }

        let mut auto_mana_events = Vec::new();
        // CR 106.6 — per-colour breakdown of what actually funded this
        // activation, threaded to the resolving body (Protective Sphere).
        let mut activation_mana_colors = Vec::new();
        if let Some(snapshot) = pre_snapshot {
            let forced_only = self.players[p].manual_mana;
            // Restricted mana may fund this only per the source's spend
            // context (e.g. ArtifactOnly mana for an artifact's ability).
            let receipt = self.try_pay_after_snapshot_mode(
                p,
                &effective_mana_cost,
                snapshot,
                forced_only,
                &ability_spend_kind,
                spend_float,
            )?;
            activation_mana_colors =
                spent_by_color(&receipt.pool_before, &self.players[p].mana_pool);
            self.pay_life_cost(p, receipt.side_effects.life_lost);
            auto_mana_events = receipt.auto_events;
        }

        // Pay the life cost. Tap and mana are committed; the life
        // payment is now safe (the pre-flight gate above guaranteed
        // sufficient life). Emits a LifeLost event so trigger / replay
        // observers see the cost.
        if ability.life_cost > 0 {
            let applied = self.adjust_life_applied(p, -(ability.life_cost as i32));
            if applied < 0 {
                auto_mana_events.push(GameEvent::LifeLost {
                    player: p,
                    amount: (-applied) as u32,
                });
            }
        }
        if ability.half_life_cost {
            let half = self.players[p].life.div_euclid(2) + self.players[p].life.rem_euclid(2);
            let applied = self.adjust_life_applied(p, -half);
            if applied < 0 {
                auto_mana_events
                    .push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
            }
        }
        if ability.x_life_cost {
            let paid = x_value.unwrap_or(0) as i32;
            let applied = self.adjust_life_applied(p, -paid);
            if applied < 0 {
                auto_mana_events.push(GameEvent::LifeLost {
                    player: p,
                    amount: (-applied) as u32,
                });
            }
        }

        // Spend the {E} cost (CR 107.16). Tap/mana/life are committed; the
        // pre-flight gate above guaranteed sufficient energy. Like the
        // `Effect::PayEnergy` spend path, no event is emitted.
        if ability.energy_cost > 0 {
            self.spend_energy(p, ability.energy_cost);
        }
        if ability.energy_x_cost {
            self.spend_energy(p, x_value.unwrap_or(0));
        }
        // Pay the collect-evidence cost (CR 701.59). Tap/mana/life/energy are
        // committed; the pre-flight gate guaranteed the graveyard can afford it.
        if let Some(amount) = ability.collect_evidence_cost {
            let mut ev = self.collect_evidence_from_graveyard(p, amount);
            auto_mana_events.append(&mut ev);
        }

        let mut events = auto_mana_events;
        // The paid tap-cost is a "becomes tapped" event (Vampire Envoy,
        // Vorinclex's opponent-land lock). Emitted after the mana payment
        // succeeded so a rolled-back activation never announces a tap.
        if ability.tap_cost {
            events.push(GameEvent::PermanentTapped { card_id, actor: None, as_attacker: false });
            // CR 605 — a mana ability's tap is also a "tapped for mana" event
            // (Extraplanar Lens), distinct from the plain tap above.
            if is_mana_ability(&ability.effect) {
                events.push(GameEvent::TappedForMana { card_id, player: p });
                if self.battlefield_find(card_id).is_some_and(|c| c.definition.is_land()) {
                    self.players[p].tapped_land_for_mana_this_turn = true;
                }
            }
        }
        if ability.untap_self_cost {
            events.push(GameEvent::PermanentUntapped { card_id });
        }
        // Mana abilities don't emit the activation event — every printed
        // "whenever an opponent activates an ability" trigger carves them
        // out (Flamescroll Celebrant, CR 605.1), and the log skips the
        // tap-for-mana spam.
        if !is_mana_ability(&ability.effect) {
            events.push(GameEvent::AbilityActivated {
                source: card_id,
                exhaust: ability.exhaust,
                adapt: ability.effect.is_adapt(),
                tap_cost: ability.tap_cost || ability.untap_self_cost,
            });
        }

        // Mark the ability as used for the once-per-turn budget. (After
        // tap/mana cost validation succeeds, before sacrifice or stack
        // queueing — all of which are guaranteed to commit if we get here.)
        if (ability.once_per_turn || ability.max_activations_per_turn.is_some())
            && !source_in_gy
            && !source_in_hand
            && let Some(card) = self.battlefield.iter_mut().find(|c| c.id == card_id)
        {
            card.once_per_turn_used.push(ability_index);
        }
        // CR 702.56 / 902.5 — record a hand (Forecast) or command-zone
        // (Vanguard) once-per-turn activation.
        if ability.once_per_turn && (source_in_hand || source_in_command) {
            self.triggered_once_per_turn_used.insert((card_id, ability_index));
        }
        // CR 603.4 — "when you next activate an exhaust ability that isn't a
        // mana ability this turn" (Pit Automaton). Claimed here so the
        // activation consumes exactly one watcher; pushed after the ability
        // itself lands on the stack so the copy resolves first.
        let exhaust_watchers: Vec<crate::game::types::DelayedTrigger> =
            if ability.exhaust && !is_mana_ability(&ability.effect) {
                let (watchers, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.delayed_triggers)
                    .into_iter()
                    .partition(|dt| {
                        dt.controller == p
                            && matches!(
                                dt.kind,
                                crate::game::types::DelayedKind::YourNextExhaustActivationThisTurn
                            )
                    });
                self.delayed_triggers = rest;
                watchers
            } else {
                Vec::new()
            };
        // CR 702.177 — record the exhaust activation (never cleared this game).
        if (ability.exhaust || ability.activate_once)
            && !source_in_gy
            && let Some(card) = self.battlefield.iter_mut().find(|c| c.id == card_id)
        {
            card.exhausted_abilities.push(ability_index);
        }

        // P/T of a creature sacrificed as part of this activation's cost,
        // re-stamped at resolution via `Effect::WithSacrificedPt` so
        // `Value::SacrificedPower/Toughness` survive intervening
        // resolutions (Witch's Oven's "toughness 4 or greater" branch).
        let mut cost_sac_pt: Option<(i32, i32)> = None;
        // The permanent the cost actually sacrificed, for
        // `Selector::SacrificedCard` at resolution.
        let mut cost_sac_card: Option<CardId> = None;
        // Mana value of the cost-sacrificed permanent, threaded the same way
        // so the `ManaValueEqualsSacrificedPlus` search filter resolves
        // correctly at ability resolution (Transfigure → Fleshwrither).
        let mut cost_sac_mv: u32 = 0;
        // Whole-batch tally across `sac_cost` + `sac_other_picks` +
        // `sac_all_matching_cost`, threaded the same way so
        // `Value::Sacrificed{Count,TotalPower}` read the batch and not the
        // last permanent (Sword of the Ages' "total power sacrificed").
        let mut cost_sac_count: u32 = 0;
        let mut cost_sac_total_power: i32 = 0;

        // Sacrifice-as-cost: with tap and mana costs paid, sacrifice the
        // source. The effect runs/queues after, and any selectors that
        // reference the source by id will miss it on the battlefield —
        // which matches the Oracle (sac is part of the activation cost,
        // so the source is in the graveyard by the time the ability
        // resolves). Cards whose effect references self after sacrifice
        // (Greater Good's "draw cards equal to its power") need to
        // capture that data via `Effect::SacrificeAndRemember` instead.
        // CR 702.6 — "Unattach this Equipment" as a cost (Sunforger): the
        // source must be attached, and it detaches before the effect runs.
        if ability.unattach_cost {
            // On an equipment-GRANTED ability (Blinding Powder, Shuriken) the
            // source is the equipped creature, so the cost detaches the granter
            // instead (CR 702.6e).
            let detach = match self.battlefield.iter().find(|c| c.id == card_id) {
                Some(c) if c.attached_to.is_some() => Some(card_id),
                Some(_) => self
                    .battlefield
                    .iter()
                    .find(|c| {
                        c.attached_to == Some(card_id)
                            && c.definition
                                .equipped_bonus
                                .as_ref()
                                .is_some_and(|b| !b.activated_abilities.is_empty())
                    })
                    .map(|c| c.id),
                None => None,
            };
            match detach.and_then(|id| self.battlefield.iter_mut().find(|c| c.id == id)) {
                Some(c) => c.attached_to = None,
                None => return Err(GameError::SelectionRequirementViolated),
            }
        }
        // "Exile [this Equipment]" as a cost on an ability the Equipment grants
        // its bearer: the cost exiles the granter, not the payer.
        if ability.exile_attachment_cost {
            let granter = self
                .battlefield
                .iter()
                .find(|c| {
                    c.attached_to == Some(card_id)
                        && c.definition
                            .equipped_bonus
                            .as_ref()
                            .is_some_and(|b| !b.activated_abilities.is_empty())
                })
                .map(|c| c.id)
                .ok_or(GameError::SelectionRequirementViolated)?;
            self.remove_from_battlefield_to_exile(granter);
            events.push(GameEvent::PermanentExiled { card_id: granter });
        }
        // "Return N [filter] you control to their owner's hand" as a cost
        // (Floodbringer). Bounce the cheapest matches so a bot doesn't throw
        // away its best permanents.
        if let Some((filter, n)) = &ability.return_permanent_cost {
            let mut pool: Vec<(CardId, u32)> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == p
                        && self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(c.id),
                            p,
                            Some(card_id),
                        )
                })
                .map(|c| (c.id, c.definition.cost.cmc()))
                .collect();
            if (pool.len() as u32) < *n {
                return Err(GameError::SelectionRequirementViolated);
            }
            pool.sort_by_key(|(_, cmc)| *cmc);
            let ctx = crate::game::effects::EffectContext::for_ability(card_id, p, None);
            for (id, _) in pool.into_iter().take(*n as usize) {
                self.move_card_to(
                    id,
                    &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::OwnerOfMoved),
                    &ctx,
                    &mut events,
                );
            }
        }
        // "Tap N untapped [filter] you control" as a cost (Crookclaw Elder).
        // Taps the least useful matches first (lowest power) so a bot doesn't
        // tap out its best attackers to draw a card.
        if let Some((filter, n)) = &ability.tap_others_cost {
            let mut pool: Vec<(CardId, i32)> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == p
                        && !c.tapped
                        && c.id != card_id
                        && self.evaluate_requirement_static(
                            filter,
                            &Target::Permanent(c.id),
                            p,
                            Some(card_id),
                        )
                })
                .map(|c| (c.id, c.definition.power))
                .collect();
            if (pool.len() as u32) < *n {
                return Err(GameError::SelectionRequirementViolated);
            }
            pool.sort_by_key(|(_, pow)| *pow);
            for (id, _) in pool.into_iter().take(*n as usize) {
                if let Some(c) = self.battlefield_find_mut(id) {
                    c.tapped = true;
                }
                events.push(GameEvent::PermanentTapped {
                    card_id: id,
                    actor: Some(p),
                    as_attacker: false,
                });
            }
        }
        if ability.sac_cost {
            let is_creature = self.permanent_is_creature(card_id);
            // The activator is the player paying the cost; the
            // sacrifice attribution should match the controller of the
            // sacrificed permanent, which is `p` (priority player) for
            // activated abilities since you only activate abilities of
            // permanents you control.
            let sac_who = self
                .battlefield_find(card_id)
                .map(|c| c.controller)
                .unwrap_or(p);
            if is_creature {
                // Stamp the sacrificed creature's P/T on the resolution
                // scratch so downstream `Value::SacrificedPower` /
                // `Value::SacrificedToughness` reads (Witch's Cauldron's
                // "gain life equal to the sacrificed creature's
                // toughness", future Thud-style `sac_cost` activations)
                // see the right values. Same plumbing as
                // `Effect::SacrificeAndRemember`.
                let snap_pt = self
                    .battlefield_find(card_id)
                    .map(|c| (c.power(), c.toughness(), c.definition.cost.cmc(), c.clone()));
                if let Some((p_val, t_val, mv, snap)) = snap_pt {
                    self.sacrificed_power = Some(p_val);
                    self.sacrificed_toughness = Some(t_val);
                    self.sacrificed_mana_value = Some(mv);
                    cost_sac_pt = Some((p_val, t_val));
                    cost_sac_card = Some(card_id);
                    cost_sac_mv = mv;
                    cost_sac_count += 1;
                    cost_sac_total_power += p_val;
                    // Cache the dying card's snapshot so AnotherOfYours
                    // triggers and type-filter predicates fire off
                    // sacrifices even when the dying card is a token.
                    self.died_card_snapshots.insert(card_id, snap.clone());
                    // CR 603.10 — also stash it as leaves-battlefield LKI so a
                    // body that reads the sacrificed source's own counters at
                    // resolution (Twitching Doll's "Spider per counter on it")
                    // sees the last-known total; `died_card_snapshots` is
                    // cleared after event dispatch, but `leaves_bf_lki` lives
                    // until the ability resolves (scoped in `resolve_stack_item`).
                    self.leaves_bf_lki.insert(card_id, snap);
                }
                // CR 701.16 — emit the sacrifice-specific event first.
                events.push(GameEvent::CreatureSacrificed { card_id, who: sac_who });
                events.push(GameEvent::CreatureDied { card_id });
            } else if let Some(snap) = self.battlefield_find(card_id).cloned() {
                // CR 603.10 — a noncreature `sac_cost` source whose body reads
                // its own last-known counters at resolution (Ratchet Bomb's
                // "destroy each nonland with mana value = charge counters")
                // needs the same leaves-battlefield LKI stash; counters are
                // stripped on the move to the graveyard (CR 122.2).
                self.died_card_snapshots.insert(card_id, snap.clone());
                self.leaves_bf_lki.insert(card_id, snap);
            }
            // Generic permanent-sacrifice event (CR 701.16) — fires for
            // every sacrificed permanent regardless of type so
            // "whenever you sacrifice a permanent" payoffs catch
            // artifact / enchantment / land sacrifices alongside
            // creatures.
            events.push(GameEvent::PermanentSacrificed { card_id, who: sac_who });
            let mut die_evs = self.remove_to_graveyard_with_triggers(card_id);
            events.append(&mut die_evs);
        }
        // Bounce-self-as-cost (CR 602.5b "…and return it to its owner's hand:"
        // — Magosi, the Waterveil). Paid after tap/mana/counter costs, so the
        // ability is already on its way to the stack when the source leaves.
        if ability.bounce_self_cost
            && let Some(snap) = self.battlefield_find(card_id).cloned()
        {
            self.leaves_bf_lki.insert(card_id, snap);
            let mut evs = self.remove_from_battlefield_to_hand(card_id);
            events.append(&mut evs);
        }

        // Return-self-as-cost: with tap/mana/life paid, bounce the source
        // back to its owner's hand (CR 602.5b). Applied before the effect
        // resolves, mirroring `sac_cost`. The mana ability / spell-copy then
        // runs with the source already in hand (Grinning Ignus, Rootha).
        if ability.return_self_cost
            && let Some(owner) = self.battlefield_find(card_id).map(|c| c.owner)
        {
            let ctx = crate::game::effects::EffectContext::for_spell(owner, None, 0, 0);
            self.move_card_to(
                card_id,
                &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::Seat(owner)),
                &ctx,
                &mut events,
            );
        }

        // "Exile a [filter] you control:" as a cost (Food Chain). Stamps the
        // last exiled permanent's mana value for `Value::ExiledForCostManaValue`.
        self.exiled_for_cost_mana_value = None;
        for cid in exile_permanent_picks {
            let mv = self.battlefield_find(cid).map(|c| c.definition.cost.cmc() as i32);
            self.move_card_to(
                cid,
                &crate::effect::ZoneDest::Exile,
                &crate::game::effects::EffectContext::for_ability(card_id, p, None),
                &mut events,
            );
            if mv.is_some() {
                self.exiled_for_cost_mana_value = mv;
            }
        }

        // "Sacrifice all [filter] you control" as a cost (Tomb of Urami) —
        // folded into the same payment loop as `sac_other_filter`. The source
        // is excluded here; pair with `sac_cost` when it also goes.
        if let Some(filter) = ability.sac_all_matching_cost.as_ref() {
            let extra: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.id != card_id && c.controller == p)
                .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
                .map(|c| c.id)
                .collect();
            sac_other_picks.extend(extra);
        }

        // Sacrifice-another-from-bf-as-cost: with tap/mana/life paid,
        // sacrifice each cost-picked battlefield permanent (already
        // validated to exist via the pre-flight `sac_other_picks`
        // lookup). Used by cards like Greater Good's `{0}, Sacrifice a
        // creature: …`, Korlash, Heir to Blackblade's `{B}, Sacrifice a
        // Swamp: …`, Witherbloom Harvester-style "sac another creature
        // for an effect" activations.
        // CR 602.5b — the whole cost-sacrifice batch feeds
        // `Value::Sacrificed{Count,TotalPower}` (Sword of the Ages sacrifices
        // any number of creatures and reads their total power). Seed from the
        // source's own `sac_cost` so an earlier resolution's tally can't leak
        // in but this activation's does.
        self.sacrificed_count = cost_sac_count;
        self.sacrificed_total_power = cost_sac_total_power;
        self.cost_sacrificed_batch = if ability.sac_cost { vec![card_id] } else { Vec::new() };
        self.cost_sacrificed_batch.extend(sac_other_picks.iter().copied());
        for other_cid in sac_other_picks {
            let sac_power = self.battlefield_find(other_cid).map(|c| c.power()).unwrap_or(0);
            self.sacrificed_count += 1;
            self.sacrificed_total_power += sac_power;
            cost_sac_count += 1;
            cost_sac_total_power += sac_power;
            let is_creature = self
                .battlefield_find(other_cid)
                .map(|c| c.definition.is_creature())
                .unwrap_or(false);
            let sac_who = self
                .battlefield_find(other_cid)
                .map(|c| c.controller)
                .unwrap_or(p);
            // Stamp the sacrificed permanent's P/T and mana value on the
            // resolution scratch so downstream `Value::SacrificedManaValue` /
            // `Value::SacrificedPower` read correctly — for *any* sacrificed
            // permanent, not just creatures (Memorial Vault — "exile 1 + the
            // sacrificed artifact's mana value").
            let snap_pt = self
                .battlefield_find(other_cid)
                .map(|c| (c.power(), c.toughness(), c.definition.cost.cmc(), c.clone()));
            if let Some((p_val, t_val, mv, snap)) = snap_pt {
                self.sacrificed_power = Some(p_val);
                self.sacrificed_toughness = Some(t_val);
                self.sacrificed_mana_value = Some(mv);
                self.sacrificed_was_artifact = Some(snap.definition.is_artifact());
                self.sacrificed_was_vehicle = Some(snap.definition.is_vehicle());
                self.sacrificed_colors = Some(snap.definition.cost.colors());
                self.sacrificed_was_outlaw =
                    Some(crate::game::effects::card_is_outlaw(&snap));
                // `Selector::SacrificedCard` reads the cost's victim.
                self.sacrificed_card = Some(other_cid);
                cost_sac_card = Some(other_cid);
                cost_sac_pt = Some((p_val, t_val));
                cost_sac_mv = mv;
                self.died_card_snapshots.insert(other_cid, snap);
            }
            if is_creature {
                events.push(GameEvent::CreatureSacrificed { card_id: other_cid, who: sac_who });
                events.push(GameEvent::CreatureDied { card_id: other_cid });
            }
            events.push(GameEvent::PermanentSacrificed { card_id: other_cid, who: sac_who });
            let mut die_evs = self.remove_to_graveyard_with_triggers(other_cid);
            events.append(&mut die_evs);
        }

        // Remove-counter-as-cost (CR 602.5b): with tap/mana/life paid, strip
        // the cost-picked counters off the source (validated by the pre-flight
        // gate). Walking Ballista's `Remove a +1/+1 counter from this: deal 1
        // damage` runs here before the ping resolves.
        if let Some((kind, count)) = ability.remove_counter_cost
            && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
        {
            let ctrl = c.controller;
            c.remove_counters(kind, count);
            if kind == crate::card::CounterType::Oil {
                self.players[ctrl].oil_activity_this_turn = true;
            }
        }

        // Remove-ALL-counters-as-cost: strip them and stamp the tally so the
        // body's `Value::CountersRemovedAsCost` can scale off it.
        self.counters_removed_as_cost = 0;
        if let Some(kind) = ability.remove_all_counters_cost
            && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
        {
            let had = c.counter_count(kind);
            c.remove_counters(kind, had);
            self.counters_removed_as_cost = had;
        }

        // Tap-permanents-as-cost (CR 602.5b): tap the pre-flight picks now that
        // tap/mana/counter payments have succeeded (Lullmage Mentor).
        for id in &tap_cost_picks {
            if let Some(c) = self.battlefield_find_mut(*id) {
                c.tapped = true;
                events.push(GameEvent::PermanentTapped {
                    card_id: *id,
                    actor: Some(p),
                    as_attacker: false,
                });
            }
        }

        // Add-counter-as-cost (CR 602.5b "Put a verse counter on this:"):
        // paid before the ability goes on the stack, so a body reading the
        // source's counters sees the new total. Yisan.
        if let Some((kind, count)) = ability.add_counter_cost
            && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
        {
            c.add_counters(kind, count);
            events.push(GameEvent::CounterAdded {
                card_id,
                counter_type: kind,
                count,
            });
        }

        // Exile-top-of-library-as-cost (CR 602.5b — Arc-Slogger). Paid after
        // tap/mana, before the ability resolves; a short library exiles what
        // it has.
        if ability.exile_top_cost > 0 {
            for _ in 0..ability.exile_top_cost {
                if self.players[p].library.is_empty() {
                    break;
                }
                let card = self.players[p].library.remove(0);
                self.place_card_in_dest(card, p, &crate::effect::ZoneDest::Exile, &mut events);
            }
        }

        // Remove-X-counters-as-cost (Arcbound Javelineer): strip the paid X.
        if let Some(kind) = ability.remove_counter_x
            && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == card_id)
        {
            c.remove_counters(kind, x_value.unwrap_or(0));
        }

        // Remove-counters-from-among-cost (Hopeful Initiate): drain `count`
        // counters distributed across matching permanents you control. The
        // auto-picker takes them lowest-power-first (validated pre-flight).
        if let Some((kinds, count, filter)) = counter_drain_cost(&ability) {
            let mut picks: Vec<(CardId, i32)> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && drainable_counters(c, kinds.as_deref()) > 0)
                .filter(|c| self.evaluate_requirement_static(&filter, &Target::Permanent(c.id), p, Some(card_id)))
                .map(|c| (c.id, c.power()))
                .collect();
            // Weakest first, but the source itself last: an ability whose body
            // feeds the source (Spike Rogue) would otherwise auto-pay itself
            // for a guaranteed no-op.
            picks.sort_by_key(|(cid, pw)| (*cid == card_id, *pw));
            let mut left = count;
            for (cid, _) in picks {
                if left == 0 { break; }
                if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) {
                    // Drain each eligible kind in turn until the quota is met.
                    let present: Vec<_> = match kinds.as_deref() {
                        Some(ks) => ks.iter().copied().filter(|k| c.counter_count(*k) > 0).collect(),
                        None => c.counters.iter().filter(|(_, n)| **n > 0).map(|(k, _)| *k).collect(),
                    };
                    for k in present {
                        if left == 0 { break; }
                        let take = left.min(c.counter_count(k));
                        c.remove_counters(k, take);
                        left -= take;
                    }
                }
            }
        }

        // Variable remove-counters-from-among cost (Ooze Flux): drain `x_value`
        // counters of the named kind across matching permanents, lowest-power
        // first (validated pre-flight). X is available to the body via XFromCost.
        if let Some((kind, filter)) = ability.remove_counter_among_x.clone() {
            let mut picks: Vec<(CardId, i32)> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == p && c.counter_count(kind) > 0)
                .filter(|c| self.evaluate_requirement_static(&filter, &Target::Permanent(c.id), p, Some(card_id)))
                .map(|c| (c.id, c.power()))
                .collect();
            picks.sort_by_key(|(cid, pw)| (*cid == card_id, *pw));
            let mut left = x_value.unwrap_or(0);
            for (cid, _) in picks {
                if left == 0 { break; }
                if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) {
                    let take = left.min(c.counter_count(kind));
                    c.remove_counters(kind, take);
                    left -= take;
                }
            }
        }

        // Discard-as-cost (CR 602.5b): with tap/mana/life paid, discard each
        // cost-picked hand card (validated via the `discard_picks` pre-flight).
        // Fauna Shaman's "Discard a creature card:" cost runs here.
        let mut discarded_for_cost_mv = None;
        let mut discarded_for_cost = 0u32;
        for cid in discard_picks {
            let mv = self.players[p]
                .hand
                .iter()
                .find(|c| c.id == cid)
                .map(|c| c.definition.cost.cmc());
            if self.discard_card(p, cid, &mut events) {
                discarded_for_cost += 1;
            }
            discarded_for_cost_mv = discarded_for_cost_mv.max(mv);
        }
        // CR 701.9 — the batch fires for a cost payment too; nothing routes a
        // cost through `resolve_effect`, so emit it here.
        if discarded_for_cost > 0 {
            events.push(GameEvent::DiscardedBatch { player: p, count: discarded_for_cost });
        }
        // Slumbering Tora reads the cost-discarded card's mana value at
        // resolution; `last_discarded_mana_value` is per-resolution scratch and
        // would be cleared before the body runs.
        self.cost_discarded_mana_value = discarded_for_cost_mv;

        // Process-as-cost: the pre-flight-picked exile cards go to their
        // owners' graveyards (CR 614.6 hate redirects still apply).
        for cid in process_picks {
            if let Some(card) = Self::take_card(&mut self.exile, cid) {
                self.route_to_graveyard(card, &mut events);
            }
        }

        // Discard-your-hand-as-cost (Diamond Lion / Lion's Eye Diamond).
        if ability.discard_hand_cost {
            let hand: Vec<CardId> = self.players[p].hand.iter().map(|c| c.id).collect();
            let mut dumped = 0u32;
            for cid in hand {
                if self.discard_card(p, cid, &mut events) {
                    dumped += 1;
                }
            }
            if dumped > 0 {
                events.push(GameEvent::DiscardedBatch { player: p, count: dumped });
            }
        }

        // Tap-another-as-cost (CR 602.5b): with tap/mana/life paid, tap the
        // pre-selected untapped permanent. Opposition's "Tap an untapped
        // creature you control" cost runs here. Capture its power first so a
        // Station ability (CR 702.184a) can stamp the counter count at
        // resolution via `Effect::WithTappedPower`.
        let mut tap_other_power: Option<i32> = None;
        if let Some(other_cid) = tap_other_pick
            && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == other_cid)
        {
            tap_other_power = Some(c.power());
            c.tapped = true;
        }

        // Tap-N-as-cost (CR 602.5b): with tap/mana/life paid, tap each
        // pre-selected untapped permanent. Heritage Druid's "Tap three
        // untapped Elves you control" cost runs here.
        self.tapped_for_cost = tap_n_picks.clone();
        for other_cid in tap_n_picks {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == other_cid) {
                c.tapped = true;
            }
        }

        // Exile-a-spell-you-control-as-cost (CR 602.5b): with tap/mana/life
        // paid, pull the cost-picked spell off the stack and exile it — it
        // won't resolve. Nivmagus Elemental.
        if let Some(spell_id) = exile_spell_pick
            && let Some(pos) = self.stack.iter().position(|item| {
                matches!(item, StackItem::Spell { card, .. } if card.id == spell_id)
            })
            && let StackItem::Spell { card, .. } = self.stack.remove(pos)
        {
            // The spell leaves the stack without resolving; it isn't
            // "countered" (no counter-a-spell payoff should fire).
            self.exile.push(*card);
            self.players[p].cards_exiled_this_turn =
                self.players[p].cards_exiled_this_turn.saturating_add(1);
        }

        // Return-another-to-hand-as-cost (CR 602.5b): with tap/mana/life paid,
        // bounce each cost-picked permanent to its owner's hand. Quirion
        // Ranger, Wirewood Symbiote.
        for other_cid in bounce_other_picks {
            self.move_card_to(
                other_cid,
                &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::OwnerOfMoved),
                &crate::game::effects::EffectContext::for_ability(card_id, p, None),
                &mut events,
            );
        }

        // Craft-exile-as-cost (CR 702.169): with tap/mana/life paid, exile
        // each cost-picked object. Battlefield permanents route through
        // `move_card_to` so leaves-the-battlefield triggers fire; graveyard
        // cards move straight to exile. Validated by the pre-flight
        // `craft_exile_picks` gate.
        for (other_cid, in_gy) in craft_exile_picks {
            if in_gy {
                if let Some(card) = Self::take_card(&mut self.players[p].graveyard, other_cid) {
                    self.exile.push(card);
                    events.push(GameEvent::CardLeftGraveyard { player: p, card_id: other_cid });
                    self.players[p].cards_left_graveyard_this_turn =
                        self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
                }
                self.players[p].cards_exiled_this_turn =
                    self.players[p].cards_exiled_this_turn.saturating_add(1);
            } else {
                self.move_card_to(
                    other_cid,
                    &crate::effect::ZoneDest::Exile,
                    &crate::game::effects::EffectContext::for_ability(card_id, p, None),
                    &mut events,
                );
            }
        }

        // Exile-another-from-gy-as-cost: with tap/mana/life paid, exile
        // each cost-picked graveyard card (already validated to exist via
        // the pre-flight `exile_other_picks` lookup). Used by cards like
        // Postmortem Professor's `{1}{B}, Exile an instant or sorcery
        // card from your graveyard: …` (count 1), Lorehold Pledgemage's
        // `{2}{R}{W}, Exile a card from your graveyard: +1/+1 EOT`
        // (count 1), and Grim Lavamancer's `{R}, {T}, Exile two cards
        // from your graveyard` (count 2).
        self.cost_exiled_cards.clear();
        for other_cid in exile_other_picks {
            if let Some(card) = Self::take_card(&mut self.players[p].graveyard, other_cid) {
                self.exile.push(card);
                self.cost_exiled_cards.push(other_cid);
                self.players[p].cards_exiled_this_turn = self.players[p]
                    .cards_exiled_this_turn
                    .saturating_add(1);
                events.push(GameEvent::CardLeftGraveyard {
                    player: p,
                    card_id: other_cid,
                });
                self.players[p].cards_left_graveyard_this_turn = self.players[p]
                    .cards_left_graveyard_this_turn
                    .saturating_add(1);
            }
        }

        // Exile-a-card-from-hand-as-cost: stamp `exiled_with` so the body can
        // read the exiled card back (Holistic Wisdom's shared-card-type gate).
        if let Some(hand_cid) = exile_from_hand_pick
            && let Some(mut card) = Self::take_card(&mut self.players[p].hand, hand_cid)
        {
            card.exiled_with = Some(card_id);
            self.exile.push(card);
            self.players[p].cards_exiled_this_turn =
                self.players[p].cards_exiled_this_turn.saturating_add(1);
            events.push(GameEvent::PermanentExiled { card_id: hand_cid });
        }

        // Exile-self-as-cost (graveyard activations): with tap/mana/life
        // paid, exile the source from the graveyard. This is the cost
        // line for cards like Stone Docent and Eternal Student that read
        // "Exile this card from your graveyard:". The effect then
        // resolves *after* the source is in exile, mirroring `sac_cost`
        // for battlefield sources.
        if ability.exile_self_cost && source_in_gy {
            let owner = source_owner.unwrap();
            if let Some(mut card) = Self::take_card(&mut self.players[owner].graveyard, card_id) {
                card.controller = owner;
                self.exile.push(card);
                self.players[owner].cards_exiled_this_turn = self.players[owner]
                    .cards_exiled_this_turn
                    .saturating_add(1);
                // Emit CardLeftGraveyard so Lorehold "cards leave your gy" payoffs fire.
                events.push(GameEvent::CardLeftGraveyard { player: owner, card_id });
                self.players[owner].cards_left_graveyard_this_turn = self.players[owner]
                    .cards_left_graveyard_this_turn
                    .saturating_add(1);
            }
        }

        // Exile-self-as-cost (hand activations): the "Exile this card from
        // your hand:" cost line of the Spirit Guides. Exile happens after
        // the (typically empty) mana cost is paid, before the mana ability
        // resolves.
        if ability.exile_self_cost && source_in_hand {
            let owner = source_owner.unwrap();
            if let Some(mut card) = Self::take_card(&mut self.players[owner].hand, card_id) {
                card.controller = owner;
                self.exile.push(card);
                self.players[owner].cards_exiled_this_turn = self.players[owner]
                    .cards_exiled_this_turn
                    .saturating_add(1);
            }
        }

        // Exile-self-as-cost (battlefield activations): the "Exile this
        // creature:" cost line on a permanent (Hanged Executioner). With
        // tap/mana/life paid, route the source battlefield → exile via the
        // shared move funnel so linked-exile returns / leaves triggers /
        // combat removal stay consistent, mirroring `return_self_cost`.
        if ability.exile_self_cost
            && !source_in_gy
            && !source_in_hand
            && let Some(owner) = self.battlefield_find(card_id).map(|c| c.owner)
        {
            let ctx = crate::game::effects::EffectContext::for_spell(owner, None, 0, 0);
            self.move_card_to(card_id, &crate::effect::ZoneDest::Exile, &ctx, &mut events);
        }

        // Put-a-card-on-top-as-cost: the pre-flight pick leaves the hand once
        // the rest of the cost is paid.
        if let Some(cid) = library_top_pick
            && let Some(card) = Self::take_card(&mut self.players[p].hand, cid)
        {
            self.players[p].library.insert(0, card);
        }

        // Discard-self-as-cost (hand activations): the "Discard this card:"
        // cost line of Elemental Masterpiece. Routes hand → graveyard via the
        // shared discard path (CardDiscarded event, madness, etc.) after mana
        // payments succeed but before the effect resolves.
        if ability.discard_self_cost && source_in_hand {
            let owner = source_owner.unwrap();
            self.discard_card(owner, card_id, &mut events);
        }

        // Mana abilities resolve immediately (no stack, no priority reset).
        let is_mana_ab = is_mana_ability(&ability.effect);

        if is_mana_ab {
            // Carry a cost-sacrificed permanent's stats into the inline
            // resolution (resolve_effect resets the scratch) — Slobad's
            // "add {R} equal to the sacrificed artifact's mana value".
            let effect = match cost_sac_pt {
                Some((power, toughness)) => Effect::WithSacrificedPt {
                    power,
                    total_power: cost_sac_total_power,
                    toughness,
                    count: cost_sac_count,
                    mana_value: cost_sac_mv,
                    card: cost_sac_card,
                    body: Box::new(ability.effect.clone()),
                },
                None => ability.effect.clone(),
            };
            // CR 701.10f / 614.5 — "tap a permanent for mana" multipliers
            // (Mana Reflection ×2, Nyxbloom Ancient ×3). Stamp the transient
            // multiplier so the `AddMana` resolver scales pip output; clear
            // it afterward. Only tapping qualifies per the printed text.
            self.mana_production_multiplier =
                if ability.tap_cost { self.mana_production_multiplier_for(p) } else { 1 };
            let resolved = self.continue_ability_resolution_x(
                card_id,
                p,
                effect,
                target.clone(),
                x_value.unwrap_or(0),
            );
            self.mana_production_multiplier = 1;
            let mut resolved = resolved?;
            // CR 605.1b — triggered mana abilities ("Whenever a land is
            // tapped for mana, … adds …") don't use the stack; they resolve
            // here, right after the tapping ability.
            if ability.tap_cost {
                self.resolve_extra_mana_on_land_tap(card_id, p, &resolved, &mut events);
            }
            events.append(&mut resolved);
        } else {
            // Non-mana activated ability goes on the stack.
            let ability_target = target.clone();
            // Carry a cost-sacrificed creature's P/T into resolution
            // (intervening resolutions reset the scratch).
            let mut queued_effect = match cost_sac_pt {
                Some((power, toughness)) => Effect::WithSacrificedPt {
                    power,
                    total_power: cost_sac_total_power,
                    toughness,
                    count: cost_sac_count,
                    mana_value: cost_sac_mv,
                    card: cost_sac_card,
                    body: Box::new(ability.effect),
                },
                None => ability.effect,
            };
            // Carry the Station-tapped creature's power into resolution
            // (CR 702.184a) so `Value::TappedForCostPower` reads it.
            if let Some(power) = tap_other_power {
                queued_effect = Effect::WithTappedPower { power, body: Box::new(queued_effect) };
            }
            // CR 601.2b — a modal activated ability's mode is chosen as part
            // of the activation (Shifting Ceratops's reach/trample/haste).
            // A submitted mode is authoritative — it's the only path by which
            // a UI seat can pick a mode whose body takes a target.
            let mode = match chosen_mode {
                Some(m) => Some(clamp_activated_mode(&queued_effect, m)),
                None => self.pick_trigger_mode(&queued_effect, card_id, p),
            };
            self.stack.push(
                TriggerPush::new(card_id, p, queued_effect)
                    .target(target)
                    .additional_targets(additional_targets.clone())
                    .mode(mode)
                    .x_value(activated_x)
                    .activated(true)
                    .mana_spent_by_color(activation_mana_colors)
                    .build(),
            );
            self.randomize_single_target_on_stack();
            // Pit Automaton — the claimed exhaust watchers go above the
            // ability they copy, so each resolves before its original.
            for dt in &exhaust_watchers {
                self.stack.push(
                    TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                        .trigger_source(Some(crate::game::effects::EntityRef::Permanent(card_id)))
                        .build(),
                );
            }
            // CR 702.21: Ward also fires on activated abilities targeting
            // an opp's Ward permanent (the "or ability" half of 702.21a).
            // Push Ward triggers above the just-queued ability so they
            // resolve first.
            self.push_ward_triggers_for_activated_ability(p, card_id, ability_target.clone());
            // BecameTarget — fire per permanent target the activation
            // chose (CR 603.x). The unified dispatcher handles APNAP and
            // the trigger filter.
            if let Some(Target::Permanent(target_id)) = &ability_target {
                let evs =
                    vec![GameEvent::BecameTarget { target: *target_id, caster: p, by: Some(card_id) }];
                self.dispatch_triggers_for_events(&evs);
            }
            // CR 601.2c — one "chose targets" event per activation. Queued
            // rather than dispatched inline: an out-of-band dispatch here
            // would drain the sacrifice-cost death events before
            // `perform_action` folds them in.
            if ability_target.is_some() || !additional_targets.is_empty() {
                events.push(GameEvent::ChoseTargets { chooser: p, object: card_id });
            }
            // CR 700.13 — activating a targeted ability against an opponent /
            // their permanents is also a crime. Queue the event alongside the
            // activation's other events (e.g. a sacrifice-cost death trigger)
            // so `perform_action` dispatches them together, in order, rather
            // than firing this out-of-band mid-activation.
            if let Some(t) = &ability_target
                && self.target_is_crime(p, t)
            {
                self.players[p].committed_crime_this_turn = true;
                events.push(GameEvent::CommittedCrime { player: p });
            }
            self.give_priority_to_active();
        }

        Ok(events)
    }
}

/// Pre-payment state captured by `snapshot_payment_state` so a failed
/// payment can revert mana pool and tap-state mutations.
pub(crate) struct PaymentSnapshot {
    pub pool: crate::mana::ManaPool,
    pub tapped: Vec<(CardId, bool)>,
}

/// What a successful payment yields: events from auto-tapping mana sources,
/// any side-effects (Phyrexian life loss), and the pool state from before
/// the payment (for convergence / similar metrics).
pub(crate) struct PaymentReceipt {
    pub auto_events: Vec<GameEvent>,
    pub side_effects: crate::mana::PaymentSideEffects,
    pub pool_before: crate::mana::ManaPool,
}

impl GameState {
    /// CR 118.8 / 119.3c — apply `life` paid as a cost (Phyrexian pips,
    /// "pay N life" riders) through the life funnel and queue the loss as
    /// a `LifeLost` event (drained after the action) so paid life fires
    /// life-loss triggers.
    pub(crate) fn pay_life_cost(&mut self, p: usize, life: u32) {
        if life == 0 {
            return;
        }
        // CR 118.8 — "whenever you pay life" (Font of Agonies) sees the amount
        // paid, whether or not the life reduction was itself replaced.
        self.pending_cost_events
            .push(GameEvent::PaidLife { player: p, amount: life });
        let applied = self.adjust_life_applied(p, -(life as i32));
        if applied < 0 {
            self.pending_cost_events
                .push(GameEvent::LifeLost { player: p, amount: (-applied) as u32 });
        }
    }

    /// `WardCost::RemoveCounterFromPermanent` — take one counter off a
    /// permanent `p` controls. Auto-pick prefers the most-stacked kind on the
    /// least valuable permanent, and never touches loyalty (removing it is a
    /// real cost the auto-payer shouldn't volunteer). Returns false — the cost
    /// is unpayable — when nothing they control carries a removable counter.
    pub(crate) fn remove_one_counter_from_own_permanent(
        &mut self,
        p: usize,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let pick = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p)
            .filter_map(|c| {
                c.counters
                    .iter()
                    .filter(|(k, n)| **n > 0 && **k != crate::card::CounterType::Loyalty)
                    .max_by_key(|(_, n)| **n)
                    .map(|(k, _)| (c.id, *k))
            })
            .next();
        let Some((cid, kind)) = pick else { return false };
        if let Some(c) = self.battlefield_find_mut(cid) {
            c.remove_counters(kind, 1);
        }
        events.push(GameEvent::CounterRemoved { card_id: cid, counter_type: kind, count: 1 });
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        true
    }
}

/// Optional-cost cast variants threaded through
/// `GameState::cast_spell_with_convoke` (Kicker / Buyback / Bestow /
/// Entwine). Each flag only sticks when the card carries the matching
/// keyword.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CastFlags {
    pub kicked: bool,
    pub buyback: bool,
    pub bestow: bool,
    pub entwine: bool,
    /// CR 709.5 — Room door being cast (`Some(0)` left, `Some(1)` right).
    pub room_door: Option<u8>,
    /// CR 702.165 — the spell's Gift was promised to an opponent.
    pub gift: bool,
    /// CR 701.67 — Waterbend: `Some(n)` adds {n} generic as an additional cost,
    /// and the `convoke_creatures` slot holds the waterbend helpers (untapped
    /// artifacts/creatures, clamped to `n`, each tapping for {1}). Stamps
    /// `cast_via_waterbend` so optional-cost riders can branch.
    pub waterbend: Option<u32>,
}


/// CR 601.2b — clamp a submitted activated-ability mode into the body's real
/// mode count. A body whose top level isn't modal ignores the pick.
fn clamp_activated_mode(effect: &crate::effect::Effect, chosen: usize) -> usize {
    match crate::game::GameState::governing_modal(effect) {
        Some(modes) if !modes.is_empty() => chosen.min(modes.len() - 1),
        _ => 0,
    }
}
