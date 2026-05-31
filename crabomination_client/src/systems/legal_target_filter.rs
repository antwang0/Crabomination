//! Client-side enumeration of legal targets for cast-time targeting.
//!
//! The engine surfaces `Decision::ChooseTarget` (with a pre-computed legal
//! list) only for triggered-ability targeting. For spells cast from hand,
//! the client just opens a targeting cursor and submits whatever the user
//! clicks — legality is validated on the server *after* the cast. That
//! leaves the highlight rings empty during the most common targeting
//! flow.
//!
//! This module fills the gap by re-running the engine's selection
//! requirement against the `ClientView` data (battlefield + players),
//! looking the card definition up in the catalog so we can read its
//! effect's slot-0 filter. The evaluator covers the predicate variants
//! that actually appear on cast-time targets — colour / mana-value / X
//! checks fall through to "permissive" so we don't accidentally hide a
//! legal target on an unsupported predicate.
//!
//! For a card that isn't in the catalog (custom decks, future formats)
//! the helper returns `None` and the caller leaves `LegalTargets`
//! unpopulated — the existing "highlight everything" fallback then
//! applies.

use std::collections::HashSet;

use crabomination::card::{CardId, SelectionRequirement};
use crabomination::game::Target;
use crabomination::net::{ClientView, PermanentView, PlayerView};

use crate::game::LegalTargets;

/// Walk the card's effect tree, pick the slot-0 filter for `mode`, and
/// evaluate it against every battlefield permanent + player. Returns
/// `None` when the card isn't found in the catalog (in which case the
/// caller should leave `LegalTargets` empty so the targeting cursor falls
/// back to its "highlight everything" behaviour).
pub fn enumerate_for_cast(
    cv: &ClientView,
    card_name: &str,
    mode: Option<usize>,
) -> Option<LegalTargets> {
    let def = crabomination::catalog::lookup_by_name(card_name)?;
    let filter = def
        .effect
        .target_filter_for_slot_in_mode(0, mode)
        .cloned()
        .unwrap_or(SelectionRequirement::Any);

    let mut out = LegalTargets {
        permanents: HashSet::new(),
        players: HashSet::new(),
        // The set is now authoritative for this card — even if it ends up
        // empty (no legal target), the highlight must not fall back to
        // "highlight everything".
        enumerated: true,
        // Source name + description are surfaced separately for engine-
        // driven `Decision::ChooseTarget`; cast-time targeting drives
        // the prompt off the hand card's KnownCard.name + the picked
        // mode's text, so leave both empty here.
        source_name: String::new(),
        description: String::new(),
    };
    for p in &cv.players {
        if evaluate_player(&filter, p, cv.your_seat) {
            out.players.insert(p.seat);
        }
    }
    for perm in &cv.battlefield {
        if evaluate_permanent(&filter, perm, cv.your_seat) {
            out.permanents.insert(perm.id);
        }
    }
    Some(out)
}

fn evaluate_player(req: &SelectionRequirement, p: &PlayerView, your_seat: usize) -> bool {
    use SelectionRequirement as R;
    match req {
        R::Any | R::Player => true,
        R::ControlledByYou => p.seat == your_seat,
        R::ControlledByOpponent => p.seat != your_seat,
        R::And(a, b) => evaluate_player(a, p, your_seat) && evaluate_player(b, p, your_seat),
        R::Or(a, b) => evaluate_player(a, p, your_seat) || evaluate_player(b, p, your_seat),
        R::Not(inner) => !evaluate_player(inner, p, your_seat),
        // Every other predicate is a permanent / card filter; players
        // can't satisfy it.
        _ => false,
    }
}

fn evaluate_permanent(
    req: &SelectionRequirement,
    perm: &PermanentView,
    your_seat: usize,
) -> bool {
    use crabomination::card::CardType;
    use SelectionRequirement as R;
    match req {
        R::Any | R::Permanent => true,
        R::Player => false,
        R::And(a, b) => {
            evaluate_permanent(a, perm, your_seat) && evaluate_permanent(b, perm, your_seat)
        }
        R::Or(a, b) => {
            evaluate_permanent(a, perm, your_seat) || evaluate_permanent(b, perm, your_seat)
        }
        R::Not(inner) => !evaluate_permanent(inner, perm, your_seat),
        R::ControlledByYou => perm.controller == your_seat,
        R::ControlledByOpponent => perm.controller != your_seat,
        R::Creature => perm.card_types.contains(&CardType::Creature),
        R::Artifact => perm.card_types.contains(&CardType::Artifact),
        R::Enchantment => perm.card_types.contains(&CardType::Enchantment),
        R::Planeswalker => perm.card_types.contains(&CardType::Planeswalker),
        R::Land => perm.card_types.contains(&CardType::Land),
        R::Nonland => !perm.card_types.contains(&CardType::Land),
        R::Noncreature => !perm.card_types.contains(&CardType::Creature),
        R::HasCardType(t) => perm.card_types.contains(t),
        R::Tapped => perm.tapped,
        R::Untapped => !perm.tapped,
        R::IsToken => perm.is_token,
        R::NotToken => !perm.is_token,
        R::HasKeyword(kw) => perm.keywords.contains(kw),
        R::PowerAtMost(n) => perm.card_types.contains(&CardType::Creature) && perm.power <= *n,
        R::PowerAtLeast(n) => perm.card_types.contains(&CardType::Creature) && perm.power >= *n,
        R::ToughnessAtMost(n) => {
            perm.card_types.contains(&CardType::Creature) && perm.toughness <= *n
        }
        R::ToughnessAtLeast(n) => {
            perm.card_types.contains(&CardType::Creature) && perm.toughness >= *n
        }
        R::WithCounter(k) => perm.counters.iter().any(|(kk, n)| kk == k && *n > 0),
        R::IsAttacking => perm.attacking,
        R::IsBlocking => perm.blocking_attacker.is_some(),
        // The cast card isn't on the battlefield yet, so the source-
        // exclusion check trivially passes.
        R::OtherThanSource => true,
        // Catalog-only predicates (mana cost, colour, supertypes,
        // subtypes, name) need the card definition. Look it up by name
        // when present, fall back to permissive otherwise.
        _ => evaluate_via_catalog(req, perm, your_seat),
    }
}

fn evaluate_via_catalog(
    req: &SelectionRequirement,
    perm: &PermanentView,
    _your_seat: usize,
) -> bool {
    use SelectionRequirement as R;
    let Some(def) = crabomination::catalog::lookup_by_name(&perm.name) else {
        // Unknown card → don't hide it from the highlight; the server's
        // check_target_legality will still reject the cast if it isn't
        // actually legal.
        return true;
    };
    match req {
        R::HasSupertype(st) => def.supertypes.contains(st),
        R::HasCreatureType(ct) => def.subtypes.creature_types.contains(ct),
        R::HasLandType(lt) => def.subtypes.land_types.contains(lt),
        R::HasArtifactSubtype(a) => def.subtypes.artifact_subtypes.contains(a),
        R::HasEnchantmentSubtype(e) => def.subtypes.enchantment_subtypes.contains(e),
        R::ManaValueAtMost(n) => def.cost.cmc() <= *n,
        R::ManaValueAtLeast(n) => def.cost.cmc() >= *n,
        // Use the cost's full color set so hybrid ({W/B}), mono-hybrid
        // ({2/R}) and Phyrexian pips contribute their color(s) — matches
        // the engine's `ManaCost::colors()` semantics rather than scanning
        // only pure `Colored` pips.
        R::HasColor(c) => def.cost.colors().contains(c),
        R::Multicolored => def.cost.distinct_colors() >= 2,
        R::Colorless => def.cost.distinct_colors() == 0,
        R::Monocolored => def.cost.distinct_colors() == 1,
        R::HasXInCost => def.cost.has_x(),
        R::IsBasicLand => def.is_land() && def
            .supertypes
            .contains(&crabomination::card::Supertype::Basic),
        R::HasName(n) => def.name == n.as_str(),
        // Anything else (HasGreatestManaValueAmongControlled, etc.) is
        // server-evaluated; default to permissive and let the server
        // reject if needed.
        _ => true,
    }
}

/// Convenience: also include a `Target::Permanent`-style helper for the
/// auto-pass / decision-target paths. Currently unused, kept for symmetry
/// with the engine's `evaluate_requirement_static` signature so callers
/// can be migrated.
#[allow(dead_code)]
pub fn evaluate(
    req: &SelectionRequirement,
    target: &Target,
    cv: &ClientView,
    your_seat: usize,
) -> bool {
    match target {
        Target::Player(s) => cv
            .players
            .iter()
            .find(|p| p.seat == *s)
            .is_some_and(|p| evaluate_player(req, p, your_seat)),
        Target::Permanent(id) => cv
            .battlefield
            .iter()
            .find(|p| p.id == *id)
            .is_some_and(|p| evaluate_permanent(req, p, your_seat)),
    }
}

/// Drop the (visual-only) entries for a card the viewer no longer
/// targets. Cheap helper for callers that want to clear the highlight
/// set without rebuilding the resource.
#[allow(dead_code)]
pub fn clear(legal: &mut LegalTargets, _: CardId) {
    legal.permanents.clear();
    legal.players.clear();
    legal.source_name.clear();
    legal.description.clear();
}
