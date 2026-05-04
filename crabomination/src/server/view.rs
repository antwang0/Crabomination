//! Per-seat projection of [`GameState`] into a [`ClientView`].
//!
//! Hides information the viewer isn't entitled to see: opponent hand contents
//! are `Hidden`; libraries surface only their size (no reveal tracking yet);
//! stack items are fully visible (no face-down spells yet). When the engine
//! gains reveal-to-seat metadata, this file is where it plugs in.

use crate::card::{CardId, CardInstance};
use crate::effect::{Effect, Value};
use crate::game::{GameState, StackItem};
use crate::mana::ManaSymbol;
use crate::net::{
    AbilityView, ClientView, ExileCardView, GraveyardCardView, HandCardView, KnownCard,
    KnownStackItem, LibraryView, PendingDecisionView, PermanentView, PlayerView, StackItemKind,
    StackItemView,
};
use crate::player::Player;

/// Project the authoritative `state` into the view visible to `seat`.
pub fn project(state: &GameState, seat: usize) -> ClientView {
    let computed = state.compute_battlefield();

    ClientView {
        your_seat: seat,
        active_player: state.active_player_idx,
        priority: state.player_with_priority(),
        step: state.step,
        turn: state.turn_number,
        players: state
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| project_player(p, i, seat))
            .collect(),
        battlefield: {
            let attacker_ids = state.attacking_ids();
            state
                .battlefield
                .iter()
                .map(|c| project_permanent(c, &computed, &attacker_ids))
                .collect()
        },
        stack: state
            .stack
            .iter()
            .map(|item| project_stack(item, state, seat))
            .collect(),
        pending_decision: state.pending_decision.as_ref().map(|pd| {
            let acting = pd.acting_player();
            PendingDecisionView {
                acting_player: acting,
                // Only the acting player sees decision specifics; spectators
                // see that someone is deciding but not the private contents.
                decision: (acting == seat).then(|| (&pd.decision).into()),
            }
        }),
        exile: state.exile.iter().map(exile_entry).collect(),
        game_over: state.game_over,
    }
}

fn exile_entry(card: &CardInstance) -> ExileCardView {
    ExileCardView {
        id: card.id,
        name: card.definition.name.to_string(),
        owner: card.owner,
    }
}

fn project_player(player: &Player, player_seat: usize, viewer_seat: usize) -> PlayerView {
    PlayerView {
        seat: player_seat,
        name: player.name.clone(),
        life: player.life,
        poison_counters: player.poison_counters,
        mana_pool: player.mana_pool.clone(),
        library: LibraryView {
            size: player.library.len(),
            known_top: Vec::new(),
        },
        graveyard: player.graveyard.iter().map(graveyard_entry).collect(),
        hand: player
            .hand
            .iter()
            .map(|c| project_hand_card(c, player_seat, viewer_seat))
            .collect(),
        lands_played_this_turn: player.lands_played_this_turn,
        first_spell_tax_charges: player.first_spell_tax_charges,
        life_gained_this_turn: player.life_gained_this_turn,
        cards_drawn_this_turn: player.cards_drawn_this_turn,
        cards_left_graveyard_this_turn: player.cards_left_graveyard_this_turn,
        creatures_died_this_turn: player.creatures_died_this_turn,
        cards_exiled_this_turn: player.cards_exiled_this_turn,
        instants_or_sorceries_cast_this_turn: player.instants_or_sorceries_cast_this_turn,
        creatures_cast_this_turn: player.creatures_cast_this_turn,
        distinct_card_types_in_graveyard: {
            let n = distinct_card_types_in_graveyard(player);
            n as u32
        },
        lifegain_prevented_this_turn: player.lifegain_prevented_this_turn,
        delirium_active: distinct_card_types_in_graveyard(player) >= 4,
    }
}

/// Count distinct `CardType`s across all cards in `player`'s graveyard.
/// Pre-computed once per `PlayerView` projection to drive both
/// `distinct_card_types_in_graveyard` and `delirium_active`.
fn distinct_card_types_in_graveyard(player: &crate::player::Player) -> usize {
    let mut seen = std::collections::HashSet::new();
    for card in &player.graveyard {
        for ct in &card.definition.card_types {
            seen.insert(ct.clone());
        }
    }
    seen.len()
}

fn project_hand_card(card: &CardInstance, owner_seat: usize, viewer_seat: usize) -> HandCardView {
    if owner_seat == viewer_seat {
        HandCardView::Known(known_card(card))
    } else {
        HandCardView::Hidden { id: card.id }
    }
}

fn known_card(card: &CardInstance) -> KnownCard {
    KnownCard {
        id: card.id,
        name: card.definition.name.to_string(),
        cost: card.definition.cost.clone(),
        card_types: card.definition.card_types.clone(),
        needs_target: card.definition.effect.requires_target(),
        has_alternative_cost: card.definition.alternative_cost.is_some(),
        back_face_name: card
            .definition
            .back_face
            .as_ref()
            .map(|b| b.name.to_string()),
        // Additional cast cost label: combines `additional_sac_cost`
        // (Daemogoth Woe-Eater, Eyeblight Cullers; push XXXIX),
        // `additional_discard_cost` (Thrilling Discovery, Cathartic
        // Reunion; push XLIII), and `additional_life_cost` (Vicious
        // Rivalry; push XLIV). When multiple are present, joins them
        // with " and ". Maps each cast-time additional cost to a
        // human-readable phrase via tiny ad-hoc renderers.
        additional_cost_label: {
            let mut parts: Vec<String> = Vec::new();
            if let Some(filter) = card.definition.additional_sac_cost.as_ref() {
                parts.push(additional_sac_cost_label(filter));
            }
            if let Some(n) = card.definition.additional_discard_cost {
                let s = additional_discard_cost_label(n);
                if parts.is_empty() {
                    parts.push(s);
                } else {
                    parts.push(s.to_lowercase());
                }
            }
            if let Some(value) = card.definition.additional_life_cost.as_ref() {
                let s = additional_life_cost_label(value);
                if parts.is_empty() {
                    parts.push(s);
                } else {
                    parts.push(s.to_lowercase());
                }
            }
            if parts.is_empty() {
                None
            } else {
                Some(parts.join(" and "))
            }
        },
        // Push XL: surface the printed `enters_with_counters` field
        // as a one-line tooltip phrase so the client can show the
        // body modification before casting. "Enters with 2 +1/+1
        // counters" / "Enters with X +1/+1 counters" / "Enters with
        // {value} {kind} counters" depending on the value shape.
        enters_with_counters_label: card
            .definition
            .enters_with_counters
            .as_ref()
            .map(|(kind, value)| enters_with_counters_label(*kind, value)),
    }
}

/// Render an `enters_with_counters` field as a one-line human label
/// for the client's card preview tooltip. Recognises the common Value
/// shapes (Const(N), XFromCost, ConvergedValue, PermanentCountControl
/// ledBy(You)) and falls back to a generic phrase for less common
/// shapes.
fn enters_with_counters_label(
    kind: crate::card::CounterType,
    value: &crate::effect::Value,
) -> String {
    use crate::effect::Value;
    let kind_label = match kind {
        crate::card::CounterType::PlusOnePlusOne => "+1/+1",
        crate::card::CounterType::MinusOneMinusOne => "-1/-1",
        crate::card::CounterType::Loyalty => "loyalty",
        crate::card::CounterType::Charge => "charge",
        crate::card::CounterType::Stun => "stun",
        crate::card::CounterType::Poison => "poison",
        _ => "counter",
    };
    let count_phrase = match value {
        Value::Const(n) => format!("{}", n),
        Value::XFromCost => "X".to_string(),
        Value::ConvergedValue => "Converge".to_string(),
        Value::PermanentCountControlledBy(_) => {
            "one per permanent you control".to_string()
        }
        _ => "N".to_string(),
    };
    format!("Enters with {} {} counters", count_phrase, kind_label)
}

/// Render an `additional_sac_cost` filter as a one-line human label.
/// The set of filters here is small (cast-time sacrifice on STX
/// `additional_sac_cost`-using cards), so an explicit shape-match is
/// sufficient. Falls back to a generic "sacrifice a permanent" when
/// the filter shape isn't recognised.
fn additional_sac_cost_label(filter: &crate::card::SelectionRequirement) -> String {
    use crate::card::SelectionRequirement as R;
    fn contains(req: &R, target: &R) -> bool {
        if std::mem::discriminant(req) == std::mem::discriminant(target) {
            return true;
        }
        match req {
            R::And(a, b) | R::Or(a, b) => contains(a, target) || contains(b, target),
            R::Not(inner) => contains(inner, target),
            _ => false,
        }
    }
    if contains(filter, &R::Creature) {
        "Sacrifice a creature".to_string()
    } else if contains(filter, &R::Artifact) {
        "Sacrifice an artifact".to_string()
    } else if contains(filter, &R::Land) {
        "Sacrifice a land".to_string()
    } else {
        "Sacrifice a permanent".to_string()
    }
}

/// Render an `additional_discard_cost` count as a one-line human label.
/// Push XLIII: pluralises "card(s)" based on the count and returns a
/// "Discard N card(s)" phrase. Used by Thrilling Discovery (1) and
/// Cathartic Reunion (2). Distinct from the resolution-time discard so
/// that the client can warn pre-cast when the controller's hand size is
/// too small to pay the cost.
fn additional_discard_cost_label(n: u32) -> String {
    if n == 1 {
        "Discard a card".to_string()
    } else {
        format!("Discard {} cards", n)
    }
}

/// Render an `additional_life_cost` Value as a one-line human label.
/// Push XLIV: surfaces the cast-time life cost so clients can warn
/// before letting a player tap into a spell that would crash them
/// below 0 life. Recognises the common Value shapes (Const, XFromCost,
/// LifeOf-style); falls back to "Pay X life" for richer shapes.
fn additional_life_cost_label(value: &crate::effect::Value) -> String {
    use crate::effect::Value;
    match value {
        Value::Const(n) => format!("Pay {} life", n),
        Value::XFromCost => "Pay X life".to_string(),
        Value::ConvergedValue => "Pay life equal to converge".to_string(),
        _ => "Pay life".to_string(),
    }
}

fn graveyard_entry(card: &CardInstance) -> GraveyardCardView {
    GraveyardCardView {
        id: card.id,
        name: card.definition.name.to_string(),
    }
}

fn project_permanent(
    card: &CardInstance,
    computed: &[crate::game::layers::ComputedPermanent],
    attacking: &[CardId],
) -> PermanentView {
    let cp = computed.iter().find(|c| c.id == card.id);
    PermanentView {
        id: card.id,
        name: card.definition.name.to_string(),
        controller: card.controller,
        owner: card.owner,
        card_types: cp
            .map(|c| c.card_types.clone())
            .unwrap_or_else(|| card.definition.card_types.clone()),
        tapped: card.tapped,
        damage: card.damage,
        summoning_sick: card.summoning_sick,
        power: cp.map(|c| c.power).unwrap_or_else(|| card.power()),
        toughness: cp.map(|c| c.toughness).unwrap_or_else(|| card.toughness()),
        base_power: card.definition.base_power(),
        base_toughness: card.definition.base_toughness(),
        keywords: cp
            .map(|c| c.keywords.clone())
            .unwrap_or_else(|| card.definition.keywords.clone()),
        counters: card.counters.iter().map(|(k, v)| (*k, *v)).collect(),
        attached_to: card.attached_to,
        is_token: card.is_token,
        attacking: attacking.contains(&card.id),
        abilities: project_abilities(card),
        loyalty_abilities: project_loyalty_abilities(card),
        loyalty: if card.definition.is_planeswalker() {
            // CR 306.5c: PW loyalty is the count of loyalty counters.
            // Cast to i32 for symmetry with the loyalty arithmetic in
            // game/mod.rs (loyalty_cost: i32, can go negative on a -X
            // ability before SBAs ground it back to 0).
            Some(
                card.counter_count(crate::card::CounterType::Loyalty) as i32,
            )
        } else {
            None
        },
        static_abilities: card
            .definition
            .static_abilities
            .iter()
            .map(|sa| sa.description.to_string())
            .collect(),
    }
}

fn project_loyalty_abilities(card: &CardInstance) -> Vec<crate::net::LoyaltyAbilityView> {
    card.definition
        .loyalty_abilities
        .iter()
        .enumerate()
        .map(|(i, a)| crate::net::LoyaltyAbilityView {
            index: i,
            loyalty_cost: a.loyalty_cost,
            effect_label: ability_effect_label(&a.effect).to_string(),
            needs_target: a.effect.requires_target(),
        })
        .collect()
}

fn project_abilities(card: &CardInstance) -> Vec<AbilityView> {
    card.definition
        .activated_abilities
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let (gate_label, gate_blocked) = match &a.condition {
                Some(p) => (predicate_short_label(p), false),
                None => (String::new(), false),
            };
            // `gate_blocked` requires evaluating the predicate against
            // the current GameState — `project_permanent` doesn't carry
            // a state reference. The view layer's caller fills this in
            // separately (see `project_permanent_with_state`); the
            // snapshot here is the static description only.
            let _ = gate_blocked;
            AbilityView {
                index: i,
                cost_label: ability_cost_label(a),
                effect_label: ability_effect_label(&a.effect).to_string(),
                needs_target: a.effect.requires_target(),
                is_mana: is_mana_ability(&a.effect),
                once_per_turn_used: a.once_per_turn && card.once_per_turn_used.contains(&i),
                gate_label,
                gate_blocked: false,
            }
        })
        .collect()
}

/// Render an `ActivatedAbility.condition` predicate as a short
/// user-facing hint string. Used to populate `AbilityView.gate_label`.
/// The format mirrors the printed Oracle text — "≥7 in hand" for hand
/// size, "spell cast this turn" for spells_cast tally, etc.
fn predicate_short_label(p: &crate::card::Predicate) -> String {
    use crate::card::Predicate;
    use crate::effect::Value;
    match p {
        Predicate::ValueAtLeast(Value::HandSizeOf(_), Value::Const(n)) => {
            format!("≥{n} in hand")
        }
        Predicate::ValueAtMost(Value::HandSizeOf(_), Value::Const(n)) => {
            format!("≤{n} in hand")
        }
        Predicate::ValueAtLeast(Value::LifeOf(_), Value::Const(n)) => {
            format!("≥{n} life")
        }
        Predicate::ValueAtMost(Value::LifeOf(_), Value::Const(n)) => {
            format!("≤{n} life")
        }
        // Push XXV: graveyard / library / count Value-keyed predicates.
        // Used by Dragon's Approach's "≥4 Dragon's Approach in gy" tutor
        // gate (`ValueAtLeast(CountOf(CardsInZone(Graveyard, HasName)),
        // Const(4))`), Resonating Lute's hand-size gate, etc. Surface a
        // short hint instead of falling through to the generic
        // "conditional" tag.
        Predicate::ValueAtLeast(Value::GraveyardSizeOf(_), Value::Const(n)) => {
            format!("≥{n} in gy")
        }
        Predicate::ValueAtMost(Value::GraveyardSizeOf(_), Value::Const(n)) => {
            format!("≤{n} in gy")
        }
        Predicate::ValueAtLeast(Value::LibrarySizeOf(_), Value::Const(n)) => {
            format!("≥{n} in library")
        }
        Predicate::ValueAtMost(Value::LibrarySizeOf(_), Value::Const(n)) => {
            format!("≤{n} in library")
        }
        // CountOf(_) compares a selector's resolved count against a
        // threshold. The selector itself is opaque from the UI's
        // perspective (could be "creatures you control", "lands of any
        // color", etc.), so the label is a structural hint —
        // "if ≥N matching" — without unpacking the selector. Same hint
        // shape as `SelectorCountAtLeast` (which targets the
        // count-on-a-zone path).
        Predicate::ValueAtLeast(Value::CountOf(_), Value::Const(1)) => {
            "if board matches".into()
        }
        Predicate::ValueAtLeast(Value::CountOf(_), Value::Const(n)) => {
            format!("if ≥{n} match")
        }
        Predicate::ValueAtMost(Value::CountOf(_), Value::Const(n)) => {
            format!("if ≤{n} match")
        }
        // Push XXX: AttackersThisCombat-keyed gates. Used by Augusta,
        // Dean of Order's "two or more creatures attack" trigger
        // (`Predicate::ValueAtLeast(AttackersThisCombat, Const(2))`)
        // and the Adriana, Captain of the Guard family. Surface a
        // short hint so UIs can preview the gate.
        Predicate::ValueAtLeast(Value::AttackersThisCombat, Value::Const(1)) => {
            "if attacking".into()
        }
        Predicate::ValueAtLeast(Value::AttackersThisCombat, Value::Const(n)) => {
            format!("if ≥{n} attackers")
        }
        Predicate::ValueAtMost(Value::AttackersThisCombat, Value::Const(n)) => {
            format!("if ≤{n} attackers")
        }
        // Push XXXI: ManaSpentToCast-keyed gates. Used by SOS Opus
        // payoffs (Tackle Artist, Spectacular Skywhale, Muse Seeker,
        // Deluge Virtuoso, Exhibition Tidecaller) — "if 5+ mana was
        // spent to cast that spell". Renders as a short readable hint.
        Predicate::ValueAtLeast(Value::ManaSpentToCast, Value::Const(n)) => {
            format!("if {n}+ mana spent")
        }
        Predicate::ValueAtMost(Value::ManaSpentToCast, Value::Const(n)) => {
            format!("if ≤{n} mana spent")
        }
        // Push XXXII: CardsDrawnThisTurn-keyed gates. Reflects the
        // "if you've drawn N cards this turn" pattern (Niv-Mizzet,
        // Parun's "first card draw" check, Sphinx's Tutelage's
        // post-draw mill). Surface a short hint so UIs can preview
        // the count threshold.
        Predicate::ValueAtLeast(Value::CardsDrawnThisTurn(_), Value::Const(1)) => {
            "after drawing".into()
        }
        Predicate::ValueAtLeast(Value::CardsDrawnThisTurn(_), Value::Const(n)) => {
            format!("if drew ≥{n}")
        }
        Predicate::ValueAtMost(Value::CardsDrawnThisTurn(_), Value::Const(n)) => {
            format!("if drew ≤{n}")
        }
        // Push XXXII: PermanentCountControlledBy-keyed gates. Reflects
        // the "X permanents you control" or "opponent controls Y
        // permanents" patterns (Possibility Storm-style). Pairs with
        // the existing `CountOf` arm, but reads off the player count
        // tally rather than a selector walk.
        Predicate::ValueAtLeast(Value::PermanentCountControlledBy(_), Value::Const(1)) => {
            "if has permanents".into()
        }
        Predicate::ValueAtLeast(Value::PermanentCountControlledBy(_), Value::Const(n)) => {
            format!("if ≥{n} permanents")
        }
        Predicate::ValueAtMost(Value::PermanentCountControlledBy(_), Value::Const(n)) => {
            format!("if ≤{n} permanents")
        }
        // EntityMatches {what, filter}: predicate over a specific entity
        // (the trigger source, target, or source-of-cast spell). The
        // Repartee filter ("trigger source matches Creature") is the
        // poster child. Push XXVI: detect common simple filters and emit
        // a more specific label — "if creature spell" / "if noncreature
        // spell" / "if artifact" — instead of the generic "if matches
        // filter" fallback. Powers Esper Sentinel ("OpponentControl +
        // EntityMatches { Noncreature }"), Felisa's death-with-counter
        // trigger filter, and any future shape-typed cast trigger.
        Predicate::EntityMatches { filter, .. } => entity_matches_label(filter),
        Predicate::SpellsCastThisTurnAtLeast { at_least: Value::Const(1), .. } => {
            "after spell cast".into()
        }
        Predicate::SpellsCastThisTurnAtLeast { at_least: Value::Const(n), .. } => {
            format!("after {n} spell casts")
        }
        Predicate::InstantsOrSorceriesCastThisTurnAtLeast {
            at_least: Value::Const(1), ..
        } => "after instant/sorcery cast".into(),
        Predicate::InstantsOrSorceriesCastThisTurnAtLeast {
            at_least: Value::Const(n), ..
        } => format!("after {n} instant/sorcery casts"),
        Predicate::CreaturesCastThisTurnAtLeast {
            at_least: Value::Const(1), ..
        } => "after creature cast".into(),
        Predicate::CreaturesCastThisTurnAtLeast {
            at_least: Value::Const(n), ..
        } => format!("after {n} creature casts"),
        Predicate::CardsLeftGraveyardThisTurnAtLeast {
            at_least: Value::Const(1), ..
        } => "after gy-leave".into(),
        Predicate::CardsLeftGraveyardThisTurnAtLeast {
            at_least: Value::Const(n), ..
        } => format!("after {n} gy-leaves"),
        Predicate::LifeGainedThisTurnAtLeast { at_least: Value::Const(1), .. } => {
            "after lifegain".into()
        }
        Predicate::LifeGainedThisTurnAtLeast { at_least: Value::Const(n), .. } => {
            format!("after {n} life gained")
        }
        Predicate::CardsExiledThisTurnAtLeast {
            at_least: Value::Const(1), ..
        } => "after exile".into(),
        Predicate::CardsExiledThisTurnAtLeast {
            at_least: Value::Const(n), ..
        } => format!("after {n} exiled"),
        Predicate::CreaturesDiedThisTurnAtLeast {
            at_least: Value::Const(1), ..
        } => "after creature death".into(),
        Predicate::CreaturesDiedThisTurnAtLeast {
            at_least: Value::Const(n), ..
        } => format!("after {n} creature deaths"),
        // Push XVI: cast-time spell-shape introspection labels.
        // Push XIX: more human-readable form for the Geometer's
        // Arthropod-style "{X} cost" trigger gate (was the more
        // technical "cast spell w/ {X}" jargon).
        Predicate::CastSpellHasX => "when you cast an X spell".into(),
        Predicate::CastSpellTargetsMatch(_) => "when target matches".into(),
        // Cast-from-graveyard (Flashback resolves): used by Antiquities
        // on the Loose's "if cast from anywhere other than your hand"
        // rider. Surface a short hint so UIs can preview the bonus.
        Predicate::CastFromGraveyard => "if cast from gy".into(),
        // Push XXIII: SelectorExists labels for "you control X" /
        // "any X exists" board-state predicates. These commonly gate
        // Social Snub's may-copy ("while you control a creature"),
        // future X-counter triggers, etc. The fallback "if X exists"
        // form is short and reads naturally in the gate badge.
        Predicate::SelectorExists(_) => "if board matches".into(),
        Predicate::SelectorCountAtLeast { n: Value::Const(1), .. } => {
            "if board matches".into()
        }
        Predicate::SelectorCountAtLeast { n: Value::Const(k), .. } => {
            format!("if ≥{k} match")
        }
        // Push XXIII: turn-of-controller label. Used by Rapier Wit's
        // "if it's your turn" stun rider and similar timing gates.
        Predicate::IsTurnOf(_) => "on your turn".into(),
        // Push XXIII: top-level And/Or short labels. The All / Any
        // boolean combinators are common as the *outer* shape of
        // chained predicate filters (Repartee = magecraft AND target-
        // is-creature). Surfacing a generic "all conditions" / "any
        // condition" form is more informative than the catch-all
        // "conditional" tag.
        Predicate::All(parts) if parts.is_empty() => "always".into(),
        Predicate::Any(parts) if parts.is_empty() => "never".into(),
        Predicate::All(_) => "all conditions".into(),
        Predicate::Any(_) => "any condition".into(),
        Predicate::Not(_) => "if not".into(),
        Predicate::True => "always".into(),
        Predicate::False => "never".into(),
        // Catch-all: no human-readable form yet.
        _ => "conditional".into(),
    }
}

/// Given a [`SelectionRequirement`] used by `Predicate::EntityMatches`,
/// produce a short human-readable label. Common simple filters get
/// dedicated labels ("if creature spell" / "if noncreature spell" /
/// "if artifact" / "if multicolored"); complex `And` / `Or` /
/// counter-keyed filters fall through to a generic "if matches filter"
/// hint. The labels read naturally in cast-trigger gate badges:
/// e.g. Esper Sentinel renders as "when opp casts spell · if noncreature
/// spell · → draw". Push XXVI helper.
fn entity_matches_label(filter: &crate::card::SelectionRequirement) -> String {
    use crate::card::SelectionRequirement as SR;
    match filter {
        SR::Creature => "if creature".into(),
        SR::Noncreature => "if noncreature".into(),
        SR::Artifact => "if artifact".into(),
        SR::Enchantment => "if enchantment".into(),
        SR::Land => "if land".into(),
        SR::Nonland => "if nonland".into(),
        SR::Planeswalker => "if planeswalker".into(),
        SR::Permanent => "if permanent".into(),
        SR::Multicolored => "if multicolored".into(),
        SR::Monocolored => "if monocolored".into(),
        SR::Colorless => "if colorless".into(),
        SR::IsBasicLand => "if basic land".into(),
        SR::IsToken => "if token".into(),
        SR::NotToken => "if non-token".into(),
        SR::IsAttacking => "if attacking".into(),
        SR::IsBlocking => "if blocking".into(),
        SR::IsSpellOnStack => "if spell".into(),
        SR::Tapped => "if tapped".into(),
        SR::Untapped => "if untapped".into(),
        SR::ControlledByYou => "if you control".into(),
        SR::ControlledByOpponent => "if opp controls".into(),
        SR::HasCardType(t) => format!("if {t:?}"),
        SR::HasColor(c) => format!("if {c}"),
        SR::HasKeyword(_) => "if has keyword".into(),
        SR::WithCounter(_) => "if has counter".into(),
        SR::PowerAtLeast(n) => format!("if power ≥{n}"),
        SR::PowerAtMost(n) => format!("if power ≤{n}"),
        SR::ToughnessAtLeast(n) => format!("if toughness ≥{n}"),
        SR::ToughnessAtMost(n) => format!("if toughness ≤{n}"),
        SR::ManaValueAtLeast(n) => format!("if MV ≥{n}"),
        SR::ManaValueAtMost(n) => format!("if MV ≤{n}"),
        SR::HasName(n) => format!("if named {n}"),
        // Push XXIX: Or-composite filters of two named card types render
        // as "if A/B" — covers Rip Apart's "creature or planeswalker"
        // and "artifact or enchantment", Nature's Claim's "artifact or
        // enchantment", Ravenous Chupacabra's "creature or planeswalker"
        // family. Recurses one level only — deeper Or chains fall back
        // to the generic hint.
        //
        // Push: 3-way Or covering creature/planeswalker/player (the
        // `effect::shortcut::any_target()` shape) collapses to
        // "any target" — same wording the printed cards use ("deals
        // 1 damage to any target"). The shortcut is left-associative
        // (`Creature.or(Planeswalker).or(Player)` →
        // `Or(Or(Creature, Planeswalker), Player)`), so we match on
        // either nesting order.
        SR::Or(outer, last)
            if matches!(last.as_ref(), SR::Player)
                && matches!(
                    outer.as_ref(),
                    SR::Or(c, p) if matches!(c.as_ref(), SR::Creature)
                        && matches!(p.as_ref(), SR::Planeswalker)
                ) =>
        {
            "any target".into()
        }
        SR::Or(first, outer)
            if matches!(first.as_ref(), SR::Player)
                && matches!(
                    outer.as_ref(),
                    SR::Or(c, p) if matches!(c.as_ref(), SR::Creature)
                        && matches!(p.as_ref(), SR::Planeswalker)
                ) =>
        {
            "any target".into()
        }
        SR::Or(a, b) => or_label(a, b),
        // Push XXX: And-composite filters where one side is
        // `IsSpellOnStack` (counter-target-spell filters) collapse to
        // the *other* side's label — the "spell" qualifier is
        // implicit when the trigger is over a spell-on-stack
        // selector. Powers Choreographed Sparks's "target IS spell
        // you control" filter rendering as "if you control" instead
        // of the generic "if matches filter". Also covers the
        // Counterspell-style "IsSpellOnStack AND (Instant OR
        // Sorcery)" — recurses into the right-hand-side via Or label.
        SR::And(a, b) if matches!(a.as_ref(), SR::IsSpellOnStack) => entity_matches_label(b),
        SR::And(a, b) if matches!(b.as_ref(), SR::IsSpellOnStack) => entity_matches_label(a),
        // Push XXX: And-composite of `ControlledByYou`/`ControlledBy
        // Opponent` plus a type token collapses to "if your X" /
        // "if opp's X" — covers `Creature ∧ ControlledByYou`,
        // `Artifact ∧ ControlledByOpponent`, etc.
        SR::And(a, b) if matches!(a.as_ref(), SR::ControlledByYou) => {
            if let Some(t) = simple_type_token(b) { format!("if your {t}") }
            else { "if matches filter".into() }
        }
        SR::And(a, b) if matches!(b.as_ref(), SR::ControlledByYou) => {
            if let Some(t) = simple_type_token(a) { format!("if your {t}") }
            else { "if matches filter".into() }
        }
        SR::And(a, b) if matches!(a.as_ref(), SR::ControlledByOpponent) => {
            if let Some(t) = simple_type_token(b) { format!("if opp's {t}") }
            else { "if matches filter".into() }
        }
        SR::And(a, b) if matches!(b.as_ref(), SR::ControlledByOpponent) => {
            if let Some(t) = simple_type_token(a) { format!("if opp's {t}") }
            else { "if matches filter".into() }
        }
        // Composite / complex predicates fall through to the generic
        // "if matches filter" hint.
        _ => "if matches filter".into(),
    }
}

/// Helper for `entity_matches_label`'s Or arm. Picks a short token for
/// each side of the Or (creature, artifact, enchantment, planeswalker,
/// land, etc.) and joins with "/". Returns the generic "if matches
/// filter" hint when either side isn't one of the simple type tokens.
fn or_label(
    a: &crate::card::SelectionRequirement,
    b: &crate::card::SelectionRequirement,
) -> String {
    let lhs = simple_type_token(a);
    let rhs = simple_type_token(b);
    match (lhs, rhs) {
        (Some(l), Some(r)) => format!("if {l}/{r}"),
        _ => "if matches filter".into(),
    }
}

/// Short type token for a `SelectionRequirement` — used by `or_label`
/// to render Or-composite filters as "if creature/planeswalker", etc.
fn simple_type_token(p: &crate::card::SelectionRequirement) -> Option<&'static str> {
    use crate::card::CardType;
    use crate::card::SelectionRequirement as SR;
    match p {
        SR::Creature => Some("creature"),
        SR::Artifact | SR::HasCardType(CardType::Artifact) => Some("artifact"),
        SR::Enchantment | SR::HasCardType(CardType::Enchantment) => Some("enchantment"),
        SR::Planeswalker | SR::HasCardType(CardType::Planeswalker) => Some("planeswalker"),
        SR::Land | SR::HasCardType(CardType::Land) => Some("land"),
        SR::HasCardType(CardType::Instant) => Some("instant"),
        SR::HasCardType(CardType::Sorcery) => Some("sorcery"),
        SR::Permanent => Some("permanent"),
        _ => None,
    }
}

fn ability_cost_label(ability: &crate::effect::ActivatedAbility) -> String {
    let mut parts: Vec<String> = Vec::new();
    for sym in &ability.mana_cost.symbols {
        // Use the Color::short_name() abbreviations so `{R}` renders
        // as `{R}` (not `{Red}`) — matches the MTG card-text format
        // the UI displays elsewhere.
        let tok = match sym {
            ManaSymbol::Colored(c) => format!("{{{c}}}"),
            ManaSymbol::Generic(n) => format!("{{{n}}}"),
            ManaSymbol::Colorless(n) => format!("{{{n}}}"),
            ManaSymbol::Hybrid(a, b) => format!("{{{a}/{b}}}"),
            ManaSymbol::Phyrexian(c) => format!("{{{c}/P}}"),
            ManaSymbol::Snow => "{S}".into(),
            ManaSymbol::X => "{X}".into(),
        };
        parts.push(tok);
    }
    if ability.tap_cost {
        parts.push("{T}".into());
    }
    // Sacrifice-cost activations (Lotus Petal, Wasteland, Mind Stone's draw
    // ability, Tormod's Crypt, …) carry a `sac_cost: true` flag — render it
    // explicitly so the UI shows the sacrifice rider rather than a label
    // that looks like the ability is free for tap+mana alone.
    if ability.sac_cost {
        parts.push("Sac".into());
    }
    // Life-cost activations (Great Hall of the Biblioplex, future
    // Phyrexian-mana flavoured activations, City of Brass-style
    // tap-for-damage hybrids). `life_cost` is the new field added
    // alongside `Effect::MayDo` in push XV.
    if ability.life_cost > 0 {
        parts.push(format!("Pay {} life", ability.life_cost));
    }
    // Push XXXIV: graveyard-exile-cost activations (Lorehold
    // Pledgemage's `{2}{R}{W}, exile a card from your graveyard:
    // +1/+1 EOT`). Render with the printed wording — matches MTG
    // card-text style.
    if ability.exile_gy_cost > 0 {
        if ability.exile_gy_cost == 1 {
            parts.push("Exile a card from your graveyard".into());
        } else {
            parts.push(format!(
                "Exile {} cards from your graveyard",
                ability.exile_gy_cost
            ));
        }
    }
    if parts.is_empty() { "0".into() } else { parts.join(", ") }
}

fn ability_effect_label(effect: &Effect) -> &'static str {
    match effect {
        Effect::AddMana { .. } => "Add mana",
        // Walk into structural combinators: pick the most representative
        // child for the label rather than degenerating to "Activate".
        Effect::Seq(steps) => {
            // Pick the most representative child: skip the catch-all
            // "Activate" placeholder, and skip a leading "Sacrifice"
            // when there's a meaningful follow-up (Goblin Bombardment,
            // Thud, Greater Good — sac is the cost; the payoff is the
            // user-facing action). If the only non-trivial step is
            // Sacrifice, fall through to that.
            let labels: Vec<&'static str> =
                steps.iter().map(ability_effect_label).collect();
            labels
                .iter()
                .copied()
                .find(|l| *l != "Activate" && *l != "Sacrifice")
                .or_else(|| labels.iter().copied().find(|l| *l != "Activate"))
                .unwrap_or("Activate")
        }
        Effect::If { then, else_, .. } => {
            // Prefer the `then` branch's label — that's the active outcome
            // when the gate passes (Gemstone Caverns luck-removal etc.).
            let lt = ability_effect_label(then);
            if lt != "Activate" { lt } else { ability_effect_label(else_) }
        }
        Effect::ChooseMode(modes) => modes
            .iter()
            .map(ability_effect_label)
            .find(|l| *l != "Activate")
            .unwrap_or("Activate"),
        Effect::ChooseModes { modes, .. } => modes
            .iter()
            .map(ability_effect_label)
            .find(|l| *l != "Activate")
            .unwrap_or("Activate"),
        Effect::PickModeAtResolution(modes) => modes
            .iter()
            .map(ability_effect_label)
            .find(|l| *l != "Activate")
            .unwrap_or("Activate"),
        Effect::ForEach { body, .. } | Effect::Repeat { body, .. } => ability_effect_label(body),
        // MayDo / MayPay wrap an inner effect — surface the inner label
        // so the UI shows what the player gets to do (the "may"
        // prompting goes through the decision panel separately).
        Effect::MayDo { body, .. } | Effect::MayPay { body, .. } => ability_effect_label(body),
        Effect::LoseLife { .. } => "Pay life / fetch land",
        Effect::Search { .. } => "Search library",
        Effect::Move { .. } => "Move permanent",
        Effect::DealDamage { .. } => "Deal damage",
        Effect::Fight { .. } => "Fight",
        Effect::Draw { .. } => "Draw cards",
        Effect::Discard { .. } => "Discard",
        Effect::Destroy { .. } => "Destroy permanent",
        Effect::Exile { .. } => "Exile permanent",
        Effect::GainLife { .. } => "Gain life",
        Effect::Mill { .. } => "Mill",
        Effect::Scry { .. } => "Scry",
        Effect::Surveil { .. } => "Surveil",
        Effect::AddCounter { .. } => "Add counter",
        Effect::RemoveCounter { .. } => "Remove counter",
        Effect::CreateToken { .. } => "Create token",
        Effect::CounterSpell { .. } => "Counter spell",
        Effect::CounterAbility { .. } => "Counter ability",
        Effect::CounterUnlessPaid { .. } => "Counter unless paid",
        Effect::Sacrifice { .. } | Effect::SacrificeAndRemember { .. } => "Sacrifice",
        Effect::DiscardChosen { .. } => "Discard chosen",
        Effect::PayOrLoseGame { .. } => "Pay or lose",
        Effect::DelayUntil { .. } => "Delayed trigger",
        Effect::Tap { .. } => "Tap",
        Effect::Untap { .. } => "Untap",
        Effect::PumpPT { power, toughness, .. } => {
            // Sign-aware split: when both terms are negative constants the
            // ability is shrinking the target rather than buffing it.
            // Burrog Befuddler ({1}{U}, magecraft -2/-0), Witherbloom
            // Command's mode 3 (-3/-3 EOT), Lash of Malice's -2/-2 EOT,
            // and Dina, Soul Steeper's activated -X/-X all read as
            // "Shrink" rather than "Pump" in the activated-ability badge
            // UI. Any non-Const value (XFromCost, CountOf, Diff) routes
            // to the conservative default of "Pump" since we can't
            // statically classify the sign at definition time.
            match (power, toughness) {
                (Value::Const(p), Value::Const(t)) if *p < 0 && *t <= 0 => "Shrink",
                (Value::Const(p), Value::Const(t)) if *p <= 0 && *t < 0 => "Shrink",
                _ => "Pump",
            }
        }
        Effect::GrantKeyword { .. } => "Grant keyword",
        Effect::AddPoison { .. } => "Add poison",
        Effect::RevealUntilFind { .. } => "Reveal until find",
        Effect::AddFirstSpellTax { .. } => "Cost tax",
        Effect::Drain { .. } => "Drain",
        Effect::Proliferate => "Proliferate",
        Effect::LookAtTop { .. } => "Look at top",
        Effect::ShuffleGraveyardIntoLibrary { .. } => "Shuffle into library",
        Effect::PutOnLibraryFromHand { .. } => "Put on library",
        Effect::RevealTopAndDrawIf { .. } => "Reveal top",
        Effect::CopySpell { .. } => "Copy spell",
        Effect::GainControl { .. } => "Gain control",
        Effect::ResetCreature { .. } => "Reset creature",
        Effect::BecomeBasicLand { .. } => "Become basic land",
        Effect::Attach { .. } => "Attach",
        Effect::GrantSorceriesAsFlash { .. } => "Sorceries as flash",
        Effect::NameCreatureType { .. } => "Name creature type",
        // `Effect::Noop` is the only Effect variant reachable here today
        // (every other variant has a dedicated arm above). The match is
        // intentionally wildcarded rather than exhaustive so adding a
        // new `Effect` variant doesn't break the build — but if a new
        // card surfaces with the generic "Activate" label, add a
        // dedicated arm above.
        _ => "Activate",
    }
}

/// "Auto-handled" mana abilities — ones that produce a fixed payload with no
/// player choice. The client filters these out of the right-click ability
/// menu (auto-tap activates them on the user's behalf). Choice-bearing mana
/// abilities like Black Lotus (`AnyOneColor`) ARE shown in the menu so the
/// player can pick a specific color before casting an off-color spell.
fn is_mana_ability(effect: &Effect) -> bool {
    use crate::effect::ManaPayload;
    match effect {
        Effect::AddMana { pool, .. } => matches!(
            pool,
            // No-choice payloads — auto-tap activates them on the user's
            // behalf without surfacing a menu entry.
            ManaPayload::Colors(_) | ManaPayload::Colorless(_)
                | ManaPayload::OfColor(_, _)
        ),
        Effect::Seq(steps) => !steps.is_empty() && steps.iter().all(is_mana_ability),
        _ => false,
    }
}

fn project_stack(item: &StackItem, state: &GameState, _viewer_seat: usize) -> StackItemView {
    match item {
        StackItem::Spell { card, caster, target, .. } => StackItemView::Known(KnownStackItem {
            source: card.id,
            controller: *caster,
            name: card.definition.name.to_string(),
            target: target.clone(),
            kind: StackItemKind::Spell,
        }),
        StackItem::Trigger { source, controller, target, .. } => {
            let name = state
                .battlefield
                .iter()
                .find(|c| c.id == *source)
                .map(|c| c.definition.name.to_string())
                .unwrap_or_else(|| "Triggered ability".to_string());
            StackItemView::Known(KnownStackItem {
                source: *source,
                controller: *controller,
                name,
                target: target.clone(),
                kind: StackItemKind::Trigger,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::game::GameState;
    use crate::net::HandCardView;
    use crate::player::Player;

    fn two_player_game() -> GameState {
        GameState::new(vec![
            Player::new(0, "P0"),
            Player::new(1, "P1"),
        ])
    }

    #[test]
    fn own_hand_is_known_opponent_hidden() {
        let mut state = two_player_game();
        state.add_card_to_hand(0, catalog::plains());
        state.add_card_to_hand(1, catalog::swamp());

        let view_p0 = project(&state, 0);
        assert!(matches!(view_p0.players[0].hand[0], HandCardView::Known(_)));
        assert!(matches!(view_p0.players[1].hand[0], HandCardView::Hidden { .. }));

        let view_p1 = project(&state, 1);
        assert!(matches!(view_p1.players[0].hand[0], HandCardView::Hidden { .. }));
        assert!(matches!(view_p1.players[1].hand[0], HandCardView::Known(_)));
    }

    #[test]
    fn library_size_public_contents_hidden() {
        let mut state = two_player_game();
        state.add_card_to_library(0, catalog::plains());
        state.add_card_to_library(0, catalog::plains());

        let view = project(&state, 1);
        assert_eq!(view.players[0].library.size, 2);
        assert!(view.players[0].library.known_top.is_empty());
    }

    #[test]
    fn stack_item_kind_distinguishes_spell_from_trigger() {
        use crate::effect::Effect;
        use crate::game::StackItem;
        let mut g = two_player_game();
        let bolt_id = g.add_card_to_battlefield(0, catalog::lightning_bolt());
        let bolt = g.battlefield_find(bolt_id).cloned().unwrap();
        g.battlefield.retain(|c| c.id != bolt_id);
        // Push one Spell and one Trigger sourced from the same card.
        g.stack.push(StackItem::Spell {
            card: Box::new(bolt),
            caster: 0,
            target: None,
            mode: None,
            x_value: 0,
            converged_value: 0,
            uncounterable: false,
            face: crate::game::types::CastFace::Front,
            is_copy: false,
        });
        g.stack.push(StackItem::Trigger {
            source: bolt_id,
            controller: 0,
            effect: Box::new(Effect::Noop),
            target: None,
            mode: None,
            x_value: 0,
            converged_value: 0,
            subject: None,
        });
        let v = project(&g, 0);
        assert_eq!(v.stack.len(), 2);
        match &v.stack[0] {
            StackItemView::Known(k) => assert_eq!(k.kind, StackItemKind::Spell),
            _ => panic!("expected Known"),
        }
        match &v.stack[1] {
            StackItemView::Known(k) => assert_eq!(k.kind, StackItemKind::Trigger),
            _ => panic!("expected Known"),
        }
    }

    #[test]
    fn exile_zone_is_public_and_includes_owner() {
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Move it to exile directly.
        let idx = state.battlefield.iter().position(|c| c.id == id).unwrap();
        let card = state.battlefield.remove(idx);
        state.exile.push(card);

        // Both seats see the exile zone identically.
        let view0 = project(&state, 0);
        let view1 = project(&state, 1);
        assert_eq!(view0.exile.len(), 1);
        assert_eq!(view0.exile[0].name, "Grizzly Bears");
        assert_eq!(view0.exile[0].owner, 0);
        assert_eq!(view1.exile.len(), 1);
        assert_eq!(view1.exile[0].name, view0.exile[0].name);
    }

    #[test]
    fn graveyard_is_public() {
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        // Move it to graveyard directly for test purposes.
        let idx = state.battlefield.iter().position(|c| c.id == id).unwrap();
        let card = state.battlefield.remove(idx);
        state.players[0].graveyard.push(card);

        let view = project(&state, 1);
        assert_eq!(view.players[0].graveyard.len(), 1);
        assert_eq!(view.players[0].graveyard[0].name, "Grizzly Bears");
    }

    /// Push XLII (CR 306.5c audit): planeswalker permanents surface
    /// their current loyalty as `PermanentView.loyalty: Option<i32>`.
    /// Non-planeswalker permanents leave the field `None`. This is a
    /// view-layer affordance — clients no longer need to scan the
    /// `counters` vec for the loyalty entry to render "Liliana 3".
    #[test]
    fn permanent_view_surfaces_planeswalker_loyalty() {
        let mut state = two_player_game();
        let liliana = state.add_card_to_battlefield(0, catalog::liliana_of_the_veil());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == liliana).unwrap();
        assert_eq!(perm.loyalty, Some(3),
            "Liliana of the Veil enters with 3 loyalty (printed)");
    }

    /// Non-planeswalker permanents leave `loyalty: None`. Back-compat:
    /// `#[serde(default)]` on the field means older serialized views
    /// continue to deserialize.
    #[test]
    fn permanent_view_leaves_loyalty_none_for_non_planeswalker() {
        let mut state = two_player_game();
        let bears = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == bears).unwrap();
        assert_eq!(perm.loyalty, None,
            "Grizzly Bears (a creature) should not have a loyalty value");
    }

    #[test]
    fn battlefield_uses_computed_power_toughness() {
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert_eq!(perm.power, 2);
        assert_eq!(perm.toughness, 2);
    }

    /// Push XXXIX: KnownCard surfaces an `additional_cost_label`
    /// for cards with `additional_sac_cost` set. Daemogoth Woe-Eater's
    /// "sacrifice a creature" cast cost should render as the printed
    /// shorthand.
    #[test]
    fn known_card_surfaces_additional_sac_cost_label() {
        let woe = catalog::daemogoth_woe_eater();
        let id = crate::card::CardId(0);
        let instance = crate::card::CardInstance::new(id, woe, 0);
        let kc = known_card(&instance);
        assert_eq!(kc.additional_cost_label.as_deref(), Some("Sacrifice a creature"),
            "Daemogoth Woe-Eater's additional sac cost should render");
    }

    /// Cards without an additional cast cost should leave the label
    /// `None` (back-compat for older serialized views).
    #[test]
    fn known_card_omits_additional_cost_label_when_none() {
        let bear = catalog::grizzly_bears();
        let id = crate::card::CardId(0);
        let instance = crate::card::CardInstance::new(id, bear, 0);
        let kc = known_card(&instance);
        assert!(kc.additional_cost_label.is_none(),
            "vanilla creatures should not surface a cost label");
    }

    /// Push XLIII: KnownCard surfaces an `additional_cost_label` for
    /// cards with `additional_discard_cost` set. Thrilling Discovery's
    /// printed "Discard a card" rider should render in the singular.
    #[test]
    fn known_card_surfaces_additional_discard_cost_label_singular() {
        let td = catalog::thrilling_discovery();
        let id = crate::card::CardId(0);
        let instance = crate::card::CardInstance::new(id, td, 0);
        let kc = known_card(&instance);
        assert_eq!(kc.additional_cost_label.as_deref(), Some("Discard a card"),
            "Thrilling Discovery's discard-1 cost should render in the singular");
    }

    /// Cathartic Reunion's discard-2 cost should render in the plural
    /// ("Discard 2 cards" — multi-card cast costs use a numeric prefix).
    #[test]
    fn known_card_surfaces_additional_discard_cost_label_plural() {
        let cr = catalog::cathartic_reunion();
        let id = crate::card::CardId(0);
        let instance = crate::card::CardInstance::new(id, cr, 0);
        let kc = known_card(&instance);
        assert_eq!(kc.additional_cost_label.as_deref(), Some("Discard 2 cards"),
            "Cathartic Reunion's discard-2 cost should render in the plural");
    }

    /// Push XLIV: KnownCard surfaces an `additional_cost_label` for
    /// cards with `additional_life_cost` set. Vicious Rivalry's printed
    /// "Pay X life" rider should render with X notation.
    #[test]
    fn known_card_surfaces_additional_life_cost_label_xfromcost() {
        let vr = catalog::vicious_rivalry();
        let id = crate::card::CardId(0);
        let instance = crate::card::CardInstance::new(id, vr, 0);
        let kc = known_card(&instance);
        assert_eq!(kc.additional_cost_label.as_deref(), Some("Pay X life"),
            "Vicious Rivalry's life-X cost should render as 'Pay X life'");
    }

    /// Push XL: KnownCard surfaces an `enters_with_counters_label`
    /// for cards with the `enters_with_counters` field set. Star
    /// Pupil's "0/0 Spirit, enters with two +1/+1 counters" should
    /// render the const-2 case as "Enters with 2 +1/+1 counters".
    #[test]
    fn known_card_surfaces_enters_with_counters_label_const() {
        let pupil = catalog::star_pupil();
        let id = crate::card::CardId(0);
        let instance = crate::card::CardInstance::new(id, pupil, 0);
        let kc = known_card(&instance);
        assert_eq!(
            kc.enters_with_counters_label.as_deref(),
            Some("Enters with 2 +1/+1 counters"),
            "Star Pupil's enters_with_counters should render as a const-2 label"
        );
    }

    /// Push XL: X-cost permanents surface "Enters with X +1/+1
    /// counters". Pterafractyl's `Value::XFromCost` branch should
    /// render the X path.
    #[test]
    fn known_card_surfaces_enters_with_counters_label_x_cost() {
        let p = catalog::pterafractyl();
        let id = crate::card::CardId(0);
        let instance = crate::card::CardInstance::new(id, p, 0);
        let kc = known_card(&instance);
        assert_eq!(
            kc.enters_with_counters_label.as_deref(),
            Some("Enters with X +1/+1 counters"),
            "Pterafractyl's enters_with_counters should render the X branch"
        );
    }

    /// Push XL: vanilla cards without `enters_with_counters` should
    /// leave the label `None` (back-compat for older serialized views).
    #[test]
    fn known_card_omits_enters_with_counters_label_when_none() {
        let bear = catalog::grizzly_bears();
        let id = crate::card::CardId(0);
        let instance = crate::card::CardInstance::new(id, bear, 0);
        let kc = known_card(&instance);
        assert!(kc.enters_with_counters_label.is_none(),
            "vanilla creatures should not surface an enters-with-counters label");
    }

    /// Sac+payoff abilities (Goblin Bombardment, Greater Good, Thud)
    /// are `Seq([Sacrifice…, Payoff…])`. The view label should surface
    /// the **payoff** so the player sees "Deal damage" / "Draw cards"
    /// rather than the cost-step "Sacrifice".
    #[test]
    fn ability_label_skips_sacrifice_cost_for_payoff() {
        let bomb = catalog::goblin_bombardment();
        let label = ability_effect_label(&bomb.activated_abilities[0].effect);
        assert_eq!(label, "Deal damage",
            "Goblin Bombardment's payoff is the user-facing action, not the sac cost");

        let good = catalog::greater_good();
        let label = ability_effect_label(&good.activated_abilities[0].effect);
        assert_eq!(label, "Draw cards",
            "Greater Good's payoff label should be Draw cards");
    }

    /// Pure-sacrifice abilities (Cankerbloom-style sac to do X) still
    /// surface "Sacrifice" — the fallback path kicks in when no
    /// non-Sacrifice non-Activate label exists.
    #[test]
    fn ability_label_falls_back_to_sacrifice_when_only_label() {
        // Build a synthetic Seq([Sacrifice]) effect — same shape as a
        // creature whose only non-mana action is to sacrifice itself.
        use crate::card::SelectionRequirement;
        use crate::effect::{Selector, Value};
        let eff = Effect::Seq(vec![Effect::Sacrifice {
            who: Selector::You,
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        }]);
        assert_eq!(ability_effect_label(&eff), "Sacrifice");
    }

    #[test]
    fn ability_cost_label_uses_mtg_color_abbreviations() {
        use crate::effect::{ActivatedAbility, Effect};
        use crate::mana::{cost, b, generic, r, w, u, x};
        let ab = ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), w(), u(), b(), r()]),
            effect: Effect::Noop,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        };
        let label = ability_cost_label(&ab);
        assert!(label.contains("{W}"), "{label} should contain {{W}}");
        assert!(label.contains("{U}"), "{label} should contain {{U}}");
        assert!(label.contains("{B}"), "{label} should contain {{B}}");
        assert!(label.contains("{R}"), "{label} should contain {{R}}");
        assert!(label.contains("{T}"), "{label} should contain the tap symbol");
        assert!(!label.contains("White") && !label.contains("Blue"),
            "label uses single-letter MTG abbreviations, not Debug names: {label}");

        let ab_x = ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[x()]),
            effect: Effect::Noop,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        };
        assert_eq!(ability_cost_label(&ab_x), "{X}",
            "X-cost ability renders as {{X}}");
    }

    /// Sacrifice-cost activated abilities (Lotus Petal, Wasteland,
    /// Tormod's Crypt, Mind Stone's draw ability) should render the
    /// "Sac" cost rider explicitly so the UI tooltip doesn't make the
    /// ability look free.
    #[test]
    fn ability_cost_label_includes_sacrifice_marker() {
        use crate::effect::{ActivatedAbility, Effect};
        use crate::mana::{cost, generic};
        // Mind Stone's draw ability: {1}, {T}, sac → Draw 1.
        let ab = ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Noop,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        };
        let label = ability_cost_label(&ab);
        assert!(label.contains("{1}"), "{label} must include the {{1}} cost");
        assert!(label.contains("{T}"), "{label} must include the tap cost");
        assert!(label.contains("Sac"),
            "{label} should advertise the sacrifice cost");

        // Lotus Petal: {T}, sac → add any one color. No mana cost.
        let petal = ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[]),
            effect: Effect::Noop,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        };
        let label = ability_cost_label(&petal);
        assert!(label.contains("{T}") && label.contains("Sac"),
            "{label} = `{{T}}, Sac`-style for Lotus Petal");
    }

    /// `AbilityView.once_per_turn_used` must reflect the engine's
    /// per-turn budget so the client can grey out the button. We set up
    /// the flag manually on the battlefield instance (rather than
    /// driving a full activation) to keep the test focused on the
    /// projection step.
    #[test]
    fn ability_view_surfaces_once_per_turn_used_state() {
        let mut state = two_player_game();
        let bio = state.add_card_to_battlefield(0, catalog::mindful_biomancer());
        // Prime the engine to "ability 0 has been used".
        state
            .battlefield
            .iter_mut()
            .find(|c| c.id == bio)
            .unwrap()
            .once_per_turn_used
            .push(0);

        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == bio).unwrap();
        let pump = perm
            .abilities
            .iter()
            .find(|a| !a.is_mana)
            .expect("Mindful Biomancer projects a non-mana pump ability");
        assert!(pump.once_per_turn_used,
            "the pump ability is once-per-turn and the engine flagged it as used");
    }

    /// Resonating Lute's gated draw ability should surface its
    /// printed `Activate only if you have seven or more cards in your
    /// hand` clause through `AbilityView.gate_label` — push VIII
    /// added the field. The client can show "≥7 in hand" next to the
    /// activator button.
    #[test]
    fn resonating_lute_gate_label_in_view() {
        let mut state = two_player_game();
        let lute = state.add_card_to_battlefield(0, catalog::resonating_lute());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == lute).unwrap();
        let draw_ability = perm.abilities.iter().find(|a| !a.is_mana)
            .expect("Resonating Lute should have a non-mana draw ability");
        assert!(!draw_ability.gate_label.is_empty(),
            "gate_label should describe the printed condition");
        assert!(draw_ability.gate_label.contains("hand"),
            "gate_label should mention 'hand' (got {:?})", draw_ability.gate_label);
    }

    /// Potioner's Trove's lifegain ability picked up a printed gate
    /// in push VIII (`SpellsCastThisTurnAtLeast(You, 1)`); the
    /// projection should expose it through `AbilityView.gate_label`.
    #[test]
    fn potioners_trove_gate_label_in_view() {
        let mut state = two_player_game();
        let trove = state.add_card_to_battlefield(0, catalog::potioners_trove());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == trove).unwrap();
        // Index 0 is mana ability; index 1 is the gated lifegain.
        let lifegain = &perm.abilities[1];
        assert_eq!(lifegain.effect_label, "Gain life");
        assert!(!lifegain.gate_label.is_empty(),
            "gate_label should describe the printed condition");
        // Push XIII: gate now uses `InstantsOrSorceriesCastThisTurnAtLeast`,
        // labeled "after instant/sorcery cast" — the test accepts either
        // wording so the label can evolve without breaking.
        let lab = &lifegain.gate_label;
        assert!(lab.contains("instant/sorcery") || lab.contains("spell"),
            "gate_label should describe the predicate (got {:?})", lab);
    }

    /// Push XXIII: predicate_short_label gained labels for
    /// `SelectorExists`, `SelectorCountAtLeast`, `IsTurnOf`, the
    /// All/Any boolean combinators, and the Not / True / False trivia.
    /// Spot-check each new arm so future refactors don't regress the
    /// labels back to the catch-all "conditional".
    #[test]
    fn predicate_short_label_covers_new_variants() {
        use crate::card::{Predicate, SelectionRequirement, Selector};
        use crate::effect::{PlayerRef, Value};

        // SelectorExists → "if board matches".
        let lab = predicate_short_label(&Predicate::SelectorExists(
            Selector::EachPermanent(SelectionRequirement::Creature),
        ));
        assert_eq!(lab, "if board matches");

        // SelectorCountAtLeast: n=1 collapses to the same; n=k formats.
        let lab = predicate_short_label(&Predicate::SelectorCountAtLeast {
            sel: Selector::EachPermanent(SelectionRequirement::Creature),
            n: Value::Const(1),
        });
        assert_eq!(lab, "if board matches");
        let lab = predicate_short_label(&Predicate::SelectorCountAtLeast {
            sel: Selector::EachPermanent(SelectionRequirement::Creature),
            n: Value::Const(3),
        });
        assert_eq!(lab, "if ≥3 match");

        // IsTurnOf → "on your turn".
        let lab = predicate_short_label(&Predicate::IsTurnOf(PlayerRef::You));
        assert_eq!(lab, "on your turn");

        // Boolean combinators.
        assert_eq!(predicate_short_label(&Predicate::All(vec![])), "always");
        assert_eq!(predicate_short_label(&Predicate::Any(vec![])), "never");
        assert_eq!(
            predicate_short_label(&Predicate::All(vec![Predicate::True])),
            "all conditions"
        );
        assert_eq!(
            predicate_short_label(&Predicate::Any(vec![Predicate::True])),
            "any condition"
        );
        assert_eq!(
            predicate_short_label(&Predicate::Not(Box::new(Predicate::True))),
            "if not"
        );

        // Trivial constants.
        assert_eq!(predicate_short_label(&Predicate::True), "always");
        assert_eq!(predicate_short_label(&Predicate::False), "never");
    }

    /// Push XXIV: predicate_short_label now formats plural N≥2 thresholds
    /// for the per-turn tally predicates (cards-left-gy, life-gained,
    /// cards-exiled, creatures-died), where previously only n=1 had a
    /// short label and n>1 fell through to "conditional".
    #[test]
    fn predicate_short_label_covers_plural_tally_thresholds() {
        use crate::card::Predicate;
        use crate::effect::{PlayerRef, Value};

        let p = Predicate::CardsLeftGraveyardThisTurnAtLeast {
            who: PlayerRef::You,
            at_least: Value::Const(3),
        };
        assert_eq!(predicate_short_label(&p), "after 3 gy-leaves");

        let p = Predicate::LifeGainedThisTurnAtLeast {
            who: PlayerRef::You,
            at_least: Value::Const(5),
        };
        assert_eq!(predicate_short_label(&p), "after 5 life gained");

        let p = Predicate::CardsExiledThisTurnAtLeast {
            who: PlayerRef::You,
            at_least: Value::Const(2),
        };
        assert_eq!(predicate_short_label(&p), "after 2 exiled");

        let p = Predicate::CreaturesDiedThisTurnAtLeast {
            who: PlayerRef::You,
            at_least: Value::Const(2),
        };
        assert_eq!(predicate_short_label(&p), "after 2 creature deaths");
    }

    /// Push XXV: predicate_short_label now covers `ValueAtLeast` /
    /// `ValueAtMost` over `GraveyardSizeOf` / `LibrarySizeOf` /
    /// `CountOf(_)` and the `EntityMatches` predicate. Spot-check each
    /// new arm so future Predicate variants don't regress these labels
    /// back to the catch-all "conditional".
    #[test]
    fn predicate_short_label_covers_value_keyed_predicates() {
        use crate::card::{Predicate, SelectionRequirement, Selector};
        use crate::effect::{PlayerRef, Value};

        // GraveyardSizeOf at-least / at-most.
        let p = Predicate::ValueAtLeast(
            Value::GraveyardSizeOf(PlayerRef::You),
            Value::Const(4),
        );
        assert_eq!(predicate_short_label(&p), "≥4 in gy");
        let p = Predicate::ValueAtMost(
            Value::GraveyardSizeOf(PlayerRef::You),
            Value::Const(2),
        );
        assert_eq!(predicate_short_label(&p), "≤2 in gy");

        // LibrarySizeOf.
        let p = Predicate::ValueAtLeast(
            Value::LibrarySizeOf(PlayerRef::You),
            Value::Const(7),
        );
        assert_eq!(predicate_short_label(&p), "≥7 in library");
        let p = Predicate::ValueAtMost(
            Value::LibrarySizeOf(PlayerRef::You),
            Value::Const(0),
        );
        assert_eq!(predicate_short_label(&p), "≤0 in library");

        // CountOf — n=1 collapses to "if board matches"; n>=2 formats.
        let creatures =
            Selector::EachPermanent(SelectionRequirement::Creature);
        let p = Predicate::ValueAtLeast(
            Value::count(creatures.clone()),
            Value::Const(1),
        );
        assert_eq!(predicate_short_label(&p), "if board matches");
        let p = Predicate::ValueAtLeast(
            Value::count(creatures.clone()),
            Value::Const(4),
        );
        assert_eq!(predicate_short_label(&p), "if ≥4 match");
        let p = Predicate::ValueAtMost(
            Value::count(creatures),
            Value::Const(2),
        );
        assert_eq!(predicate_short_label(&p), "if ≤2 match");

        // EntityMatches: push XXVI now reads the inner filter and emits a
        // type-specific label for common cases. Composite / counter-keyed
        // filters fall through to "if matches filter".
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::Creature,
        };
        assert_eq!(predicate_short_label(&p), "if creature");
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::Noncreature,
        };
        assert_eq!(predicate_short_label(&p), "if noncreature");
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::Artifact,
        };
        assert_eq!(predicate_short_label(&p), "if artifact");
        // Composite predicate falls through to the generic hint.
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::Multicolored),
        };
        assert_eq!(predicate_short_label(&p), "if matches filter");
    }

    /// Push XXIX: `entity_matches_label` now resolves Or-composite
    /// filters of two simple type tokens into "if A/B" labels.
    /// Covers Rip Apart's "creature/planeswalker" + "artifact/
    /// enchantment", Magma Opus's "creature/planeswalker", and
    /// Nature's Claim's "artifact/enchantment".
    #[test]
    fn entity_matches_label_covers_or_composite_filters() {
        use crate::card::{Predicate, SelectionRequirement, Selector};
        // Creature OR Planeswalker → "if creature/planeswalker".
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Planeswalker),
        };
        assert_eq!(predicate_short_label(&p), "if creature/planeswalker");
        // Artifact OR Enchantment → "if artifact/enchantment".
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::HasCardType(
                crate::card::CardType::Artifact,
            )
                .or(SelectionRequirement::HasCardType(
                    crate::card::CardType::Enchantment,
                )),
        };
        assert_eq!(predicate_short_label(&p), "if artifact/enchantment");
        // Instant OR Sorcery → "if instant/sorcery".
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::HasCardType(
                crate::card::CardType::Instant,
            )
                .or(SelectionRequirement::HasCardType(
                    crate::card::CardType::Sorcery,
                )),
        };
        assert_eq!(predicate_short_label(&p), "if instant/sorcery");
        // Three-way Or chain falls back to generic hint (we only
        // recurse one level deep).
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Artifact)
                .or(SelectionRequirement::Enchantment),
        };
        assert_eq!(predicate_short_label(&p), "if matches filter");

        // The `effect::shortcut::any_target()` shape (Creature ∨
        // Planeswalker ∨ Player, left-associative) renders as the
        // canonical "any target" — same wording the printed cards
        // use for "deals N damage to any target".
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Planeswalker)
                .or(SelectionRequirement::Player),
        };
        assert_eq!(predicate_short_label(&p), "any target");
    }

    /// Push XXX: `entity_matches_label` now collapses common
    /// And-composite filters too. `IsSpellOnStack ∧ X` strips the
    /// "spell" qualifier; `ControlledByYou ∧ X` / `ControlledBy
    /// Opponent ∧ X` collapse to "if your X" / "if opp's X".
    /// Covers Choreographed Sparks's "you control" stack-spell
    /// filter, Saw It Coming-style counter-target-IS filters, and
    /// any future "your creature" / "opp's artifact" matters.
    #[test]
    fn entity_matches_label_covers_and_composite_filters() {
        use crate::card::{Predicate, SelectionRequirement, Selector};
        // IsSpellOnStack ∧ ControlledByYou → "if you control"
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::IsSpellOnStack
                .and(SelectionRequirement::ControlledByYou),
        };
        assert_eq!(predicate_short_label(&p), "if you control");
        // Creature ∧ ControlledByYou → "if your creature"
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou),
        };
        assert_eq!(predicate_short_label(&p), "if your creature");
        // Artifact ∧ ControlledByOpponent → "if opp's artifact"
        let p = Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::Artifact
                .and(SelectionRequirement::ControlledByOpponent),
        };
        assert_eq!(predicate_short_label(&p), "if opp's artifact");
    }

    /// Push XXX: `predicate_short_label` now formats the new
    /// `Value::AttackersThisCombat` primitive in
    /// `Predicate::ValueAtLeast(_, Const(n))` shape — surfaces the
    /// gate as "if ≥N attackers" so Augusta, Dean of Order's
    /// "two or more creatures attack" rider reads cleanly.
    #[test]
    fn predicate_short_label_covers_attackers_this_combat() {
        use crate::card::Predicate;
        use crate::effect::Value;
        let p = Predicate::ValueAtLeast(Value::AttackersThisCombat, Value::Const(2));
        assert_eq!(predicate_short_label(&p), "if ≥2 attackers");
        let p = Predicate::ValueAtLeast(Value::AttackersThisCombat, Value::Const(1));
        assert_eq!(predicate_short_label(&p), "if attacking");
        let p = Predicate::ValueAtMost(Value::AttackersThisCombat, Value::Const(3));
        assert_eq!(predicate_short_label(&p), "if ≤3 attackers");
    }

    /// Push XXXI: `predicate_short_label` formats the new
    /// `Value::ManaSpentToCast` primitive in
    /// `Predicate::ValueAtLeast(_, Const(n))` / `ValueAtMost` shape so
    /// Opus payoffs (Tackle Artist, Spectacular Skywhale, Muse Seeker,
    /// Deluge Virtuoso, Exhibition Tidecaller) show their 5+-mana gate
    /// as "if 5+ mana spent" rather than the catch-all "conditional".
    #[test]
    fn predicate_short_label_covers_mana_spent_to_cast() {
        use crate::card::Predicate;
        use crate::effect::Value;
        let p = Predicate::ValueAtLeast(Value::ManaSpentToCast, Value::Const(5));
        assert_eq!(predicate_short_label(&p), "if 5+ mana spent");
        let p = Predicate::ValueAtMost(Value::ManaSpentToCast, Value::Const(4));
        assert_eq!(predicate_short_label(&p), "if ≤4 mana spent");
    }

    /// Push XXXII: `predicate_short_label` covers
    /// `Value::CardsDrawnThisTurn` and `Value::PermanentCountControlledBy`
    /// in the `ValueAtLeast`/`ValueAtMost` shapes — used by lifegain /
    /// permanent-count gates that don't go through `CountOf` selector
    /// walks.
    #[test]
    fn predicate_short_label_covers_cards_drawn_and_permanent_count() {
        use crate::card::Predicate;
        use crate::effect::{PlayerRef, Value};
        let p = Predicate::ValueAtLeast(Value::CardsDrawnThisTurn(PlayerRef::You), Value::Const(1));
        assert_eq!(predicate_short_label(&p), "after drawing");
        let p = Predicate::ValueAtLeast(Value::CardsDrawnThisTurn(PlayerRef::You), Value::Const(3));
        assert_eq!(predicate_short_label(&p), "if drew ≥3");
        let p = Predicate::ValueAtMost(Value::CardsDrawnThisTurn(PlayerRef::You), Value::Const(2));
        assert_eq!(predicate_short_label(&p), "if drew ≤2");

        let p = Predicate::ValueAtLeast(
            Value::PermanentCountControlledBy(PlayerRef::You), Value::Const(1));
        assert_eq!(predicate_short_label(&p), "if has permanents");
        let p = Predicate::ValueAtLeast(
            Value::PermanentCountControlledBy(PlayerRef::You), Value::Const(4));
        assert_eq!(predicate_short_label(&p), "if ≥4 permanents");
        let p = Predicate::ValueAtMost(
            Value::PermanentCountControlledBy(PlayerRef::You), Value::Const(2));
        assert_eq!(predicate_short_label(&p), "if ≤2 permanents");
    }

    /// Push: `ability_effect_label` for `Effect::PumpPT` now splits on
    /// the sign of `power`/`toughness`. Both negative → "Shrink"
    /// (Burrog Befuddler / Witherbloom Command's mode 3 / Lash of
    /// Malice / Dina, Soul Steeper). Positive or mixed (X-cost,
    /// dynamic) → "Pump". Without the split, Burrog Befuddler's
    /// magecraft -2/-0 trigger labels as "Pump" in the activated-
    /// ability badge UI — misleading flavor.
    #[test]
    fn ability_effect_label_splits_pump_vs_shrink_by_sign() {
        use crate::card::Selector;
        use crate::effect::{Duration, Value};
        let pump = Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        };
        assert_eq!(ability_effect_label(&pump), "Pump");
        let shrink_neg_power = Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(-2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        };
        assert_eq!(ability_effect_label(&shrink_neg_power), "Shrink");
        let shrink_neg_tough = Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(0),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        };
        assert_eq!(ability_effect_label(&shrink_neg_tough), "Shrink");
        let shrink_both = Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        };
        assert_eq!(ability_effect_label(&shrink_both), "Shrink");
        // X-cost / dynamic values default to "Pump" (conservative).
        let dyn_pump = Effect::PumpPT {
            what: Selector::This,
            power: Value::XFromCost,
            toughness: Value::XFromCost,
            duration: Duration::EndOfTurn,
        };
        assert_eq!(ability_effect_label(&dyn_pump), "Pump");
    }

    /// Push XXXVII: `Effect::PickModeAtResolution` should label using the
    /// first non-trivial inner mode's label (same shape as ChooseMode /
    /// ChooseModes). Prismari Apprentice's magecraft "Scry 1 or +1/+0
    /// EOT" should surface as "Pump" (the inner Scry's "Activate"
    /// fallback gets skipped) — actually scry comes first so it's
    /// "Scry/Surveil".
    #[test]
    fn ability_effect_label_handles_pick_mode_at_resolution() {
        use crate::card::Selector;
        use crate::effect::{Duration, PlayerRef, Value};
        let pmr = Effect::PickModeAtResolution(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]);
        // First non-fallback label wins. Scry → "Scry/Surveil" or similar.
        // We assert it's NOT "Activate" (the catch-all fallback) — proving
        // the new arm is exercising the inner modes.
        let label = ability_effect_label(&pmr);
        assert_ne!(label, "Activate",
            "PickModeAtResolution should surface inner mode label, not the fallback");
    }

    /// Planeswalkers' loyalty abilities should surface in the wire view so
    /// the client can render the "+1 / -3 / -8" buttons. Pre-fix the
    /// PermanentView only carried activated abilities, leaving the UI
    /// blind to walker abilities.
    #[test]
    fn planeswalker_loyalty_abilities_appear_in_view() {
        let mut state = two_player_game();
        let karn = state.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == karn).unwrap();
        assert_eq!(perm.loyalty_abilities.len(), 3, "Karn has +1, -1, -2");
        let costs: Vec<i32> = perm.loyalty_abilities.iter().map(|l| l.loyalty_cost).collect();
        assert_eq!(costs, vec![1, -1, -2]);
        // The -2 ability creates a token; pre-rendered label should reflect that.
        assert_eq!(perm.loyalty_abilities[2].effect_label, "Create token");
    }

    /// Push XXXVIII: PermanentView gains a `static_abilities`
    /// `Vec<String>` populated from each `StaticAbility.description`.
    /// Lets clients render the printed rules-text without rebuilding it
    /// from the static-effect tree. Verifies both Killian (single
    /// CostReductionTargeting static) and Hofri (single PumpPT
    /// static).
    #[test]
    fn static_abilities_descriptions_appear_in_permanent_view() {
        let mut state = two_player_game();
        let killian = state.add_card_to_battlefield(0, catalog::killian_ink_duelist());
        let hofri = state.add_card_to_battlefield(0, catalog::hofri_ghostforge());
        let view = project(&state, 0);

        let killian_view = view.battlefield.iter().find(|p| p.id == killian).unwrap();
        assert_eq!(killian_view.static_abilities.len(), 1);
        assert!(killian_view.static_abilities[0].contains("less"),
            "Killian should advertise its cost-reduction static: got `{}`",
            killian_view.static_abilities[0]);

        let hofri_view = view.battlefield.iter().find(|p| p.id == hofri).unwrap();
        assert_eq!(hofri_view.static_abilities.len(), 1);
        assert!(hofri_view.static_abilities[0].contains("+1/+1"),
            "Hofri should advertise its anthem static: got `{}`",
            hofri_view.static_abilities[0]);
    }

    /// Permanents with no static abilities surface an empty vec — back-
    /// compat with the prior `serde(default)` for older snapshots and
    /// non-static cards (Grizzly Bears, etc.).
    #[test]
    fn static_abilities_empty_for_vanilla_creature() {
        let mut state = two_player_game();
        let bear = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let view = project(&state, 0);
        let bear_view = view.battlefield.iter().find(|p| p.id == bear).unwrap();
        assert!(bear_view.static_abilities.is_empty(),
            "Vanilla bear should have empty static_abilities");
    }

    /// Push XXXIV: graveyard-exile-cost activations (Lorehold Pledgemage)
    /// surface the printed wording in the cost label. Both the `1`
    /// case (printed "exile a card") and `≥2` (printed "exile N cards")
    /// render with the printed-text wording.
    #[test]
    fn ability_cost_label_renders_exile_gy_cost() {
        use crate::effect::{ActivatedAbility, Effect};
        use crate::mana::{cost, generic, r, w};
        // Lorehold Pledgemage: {2}{R}{W}, exile-gy: +1/+1 EOT.
        let pledge = ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), r(), w()]),
            effect: Effect::Noop,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 1,
        };
        let label = ability_cost_label(&pledge);
        assert!(label.contains("Exile a card from your graveyard"),
            "Label should advertise the gy-exile cost: got `{label}`");

        // Plural form for hypothetical multi-exile cost.
        let multi = ActivatedAbility {
            exile_gy_cost: 3,
            ..pledge
        };
        let label = ability_cost_label(&multi);
        assert!(label.contains("Exile 3 cards from your graveyard"),
            "Plural form should include the count: got `{label}`");
    }
}
