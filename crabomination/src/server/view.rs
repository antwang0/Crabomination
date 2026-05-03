//! Per-seat projection of [`GameState`] into a [`ClientView`].
//!
//! Hides information the viewer isn't entitled to see: opponent hand contents
//! are `Hidden`; libraries surface only their size (no reveal tracking yet);
//! stack items are fully visible (no face-down spells yet). When the engine
//! gains reveal-to-seat metadata, this file is where it plugs in.

use crate::card::{CardId, CardInstance};
use crate::effect::Effect;
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
    }
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
        Effect::PumpPT { .. } => "Pump",
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

    #[test]
    fn battlefield_uses_computed_power_toughness() {
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert_eq!(perm.power, 2);
        assert_eq!(perm.toughness, 2);
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
}
