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
    let affordances = state.compute_hand_affordances(seat);

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
            .map(|(i, p)| {
                use crate::mana::Color;
                let devotion = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
                    .map(|c| state.devotion_to(i, &[c]).max(0) as u32);
                project_player(p, i, seat, &state.prevention_shields, devotion, state.draw_cap_for(i), state.monarch == Some(i), commander_damage_taken(state, i), state.team_of(i).0)
            })
            .collect(),
        battlefield: {
            let attacker_ids = state.attacking_ids();
            let block_map = state.block_map_snapshot();
            state
                .battlefield
                .iter()
                .map(|c| {
                    project_permanent(
                        c,
                        &computed,
                        &attacker_ids,
                        &block_map,
                        &state.prevention_shields,
                        &state.battlefield,
                    )
                })
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
        damage_cant_be_prevented_this_turn: state.damage_cant_be_prevented_this_turn,
        day_night: state.day_night.map(|dn| dn == crate::game::types::DayNight::Day),
        combat_preview: combat_preview(state),
        // One pass over the hand builds a single library-stripped probe
        // template and reuses it across every affordance category, rather
        // than each `*_hand_cards` call cloning the whole `GameState` (incl.
        // both libraries) per candidate. Runs on every accepted action for
        // the priority-holding seat, so it's the projection's hot path.
        castable_hand: affordances.castable,
        pitchable_hand: affordances.pitchable,
        kickable_hand: affordances.kickable,
        buyback_hand: affordances.buyback,
        bestowable_hand: affordances.bestowable,
        dashable_hand: affordances.dashable,
        blitzable_hand: affordances.blitzable,
        suspendable_hand: affordances.suspendable,
        foretellable_hand: affordances.foretellable,
        plottable_hand: affordances.plottable,
        adventurable_hand: affordances.adventurable,
        splittable_right_hand: affordances.splittable_right,
        activatable_permanents: affordances.activatable_permanents,
        legal_attackers: state.legal_attackers(seat),
        legal_blockers: state.legal_blockers(seat),
        permanents_to_graveyard_this_turn: state.permanents_to_graveyard_this_turn,
    }
}

/// Compute a [`CombatPreview`] from the current attacker/blocker
/// assignment. Returns `None` when no attackers are declared. See the
/// struct doc for the modeling caveats.
fn combat_preview(state: &GameState) -> Option<crate::net::CombatPreview> {
    use crate::card::Keyword;
    use crate::game::types::AttackTarget;
    let attackers = state.attacking();
    if attackers.is_empty() {
        return None;
    }
    let block_map = state.block_map_snapshot(); // (blocker, attacker)
    let mut dmg: std::collections::HashMap<usize, i32> = std::collections::HashMap::new();
    let mut lifegain: std::collections::HashMap<usize, i32> = std::collections::HashMap::new();
    let mut dying: Vec<CardId> = Vec::new();

    let lethal_from = |attacker: &crate::card::CardInstance, defender: &crate::card::CardInstance| -> bool {
        let p = attacker.power();
        p > 0
            && !defender.has_keyword(&Keyword::Indestructible)
            // CR 702.16e — combat damage from a color the defender has
            // protection from is prevented, so it never dies to that source.
            && !state.damage_prevented_by_protection(attacker.id, defender.id)
            && (p >= defender.toughness() || attacker.has_keyword(&Keyword::Deathtouch))
    };

    for atk in attackers {
        let Some(a) = state.battlefield_find(atk.attacker) else { continue };
        let blockers: Vec<&crate::card::CardInstance> = block_map
            .iter()
            .filter(|(_, aid)| *aid == atk.attacker)
            .filter_map(|(bid, _)| state.battlefield_find(*bid))
            .collect();
        let a_power = a.power().max(0);
        let lifelink = a.has_keyword(&Keyword::Lifelink);
        if blockers.is_empty() {
            // Unblocked: full damage to the defending player (planeswalker
            // targets don't hit a player's life).
            if let AttackTarget::Player(p) = atk.target {
                *dmg.entry(p).or_insert(0) += a_power;
                if lifelink {
                    *lifegain.entry(a.controller).or_insert(0) += a_power;
                }
            }
        } else {
            // Blocked: attacker assigns lethal to blockers in id order;
            // trample overflows to the defending player.
            let has_fs = |c: &crate::card::CardInstance| {
                c.has_keyword(&Keyword::FirstStrike) || c.has_keyword(&Keyword::DoubleStrike)
            };
            let attacker_fs = has_fs(a);
            // Which blockers does the attacker kill? (lethal spread, deathtouch
            // first-blocker-eats-all). Computed first so first-strike removal
            // can suppress those blockers' damage back.
            let mut remaining = a_power;
            let mut killed: Vec<CardId> = Vec::new();
            for b in &blockers {
                let needed = b.toughness().max(1);
                if lethal_from(a, b) && (a.has_keyword(&Keyword::Deathtouch) || remaining >= needed) {
                    killed.push(b.id);
                    remaining -= if a.has_keyword(&Keyword::Deathtouch) { 1 } else { needed };
                }
            }
            // CR 702.7 — a non-first-strike blocker the attacker kills in the
            // first-strike step deals no damage back. Such blockers don't
            // count toward the attacker's death.
            let deals_back =
                |b: &crate::card::CardInstance| !(attacker_fs && !has_fs(b) && killed.contains(&b.id));
            let total_blocker_power: i32 =
                blockers.iter().filter(|b| deals_back(b)).map(|b| b.power().max(0)).sum();
            let dt_blocker = blockers
                .iter()
                .any(|b| b.power() > 0 && b.has_keyword(&Keyword::Deathtouch) && deals_back(b));
            if (total_blocker_power > 0 || dt_blocker)
                && !a.has_keyword(&Keyword::Indestructible)
                && (total_blocker_power >= a.toughness() || dt_blocker)
            {
                dying.push(a.id);
            }
            for bid in &killed {
                dying.push(*bid);
            }
            // Trample overflow (CR 510.1c): leftover after lethal to all
            // blockers spills to the defending player.
            if a.has_keyword(&Keyword::Trample) {
                let assign_to_block: i32 = blockers
                    .iter()
                    .map(|b| if a.has_keyword(&Keyword::Deathtouch) { 1 } else { b.toughness().max(0) })
                    .sum();
                let overflow = (a_power - assign_to_block).max(0);
                if overflow > 0
                    && let AttackTarget::Player(p) = atk.target
                {
                    *dmg.entry(p).or_insert(0) += overflow;
                }
            }
            if lifelink {
                *lifegain.entry(a.controller).or_insert(0) += a_power;
            }
            // Blockers with lifelink gain their controller life for the
            // damage they deal to the attacker (a first-struck-dead blocker
            // deals none).
            for b in &blockers {
                if b.has_keyword(&Keyword::Lifelink) && deals_back(b) {
                    *lifegain.entry(b.controller).or_insert(0) += b.power().max(0);
                }
            }
        }
    }

    dying.sort();
    dying.dedup();
    let mut damage_to_players: Vec<(usize, i32)> = dmg.into_iter().filter(|(_, d)| *d != 0).collect();
    damage_to_players.sort();
    let mut lifegain_to_players: Vec<(usize, i32)> = lifegain.into_iter().filter(|(_, d)| *d != 0).collect();
    lifegain_to_players.sort();
    Some(crate::net::CombatPreview { damage_to_players, lifegain_to_players, dying_creatures: dying })
}

fn exile_entry(card: &CardInstance) -> ExileCardView {
    ExileCardView {
        id: card.id,
        name: card.definition.name.to_string(),
        owner: card.owner,
        may_play_recipient: card.may_play_until.as_ref().map(|p| p.player),
        mana_value: card.definition.cost.cmc(),
        is_token: card.is_token,
        exiled_by: card.exiled_by.map(|l| l.source),
    }
}

/// Collect the commander-damage tally dealt to `victim` (CR 903.10a), one
/// entry per source commander, resolving each source `CardId` to its current
/// name + owning seat. Sorted by descending damage so the closest-to-lethal
/// source leads. Empty outside Commander games.
fn commander_damage_taken(
    state: &GameState,
    victim: usize,
) -> Vec<crate::net::CommanderDamageEntry> {
    let mut entries: Vec<crate::net::CommanderDamageEntry> = state
        .commander_damage
        .iter()
        .filter(|((v, _), amount)| *v == victim && **amount > 0)
        .map(|((_, source_id), amount)| {
            // A commander source is usually on the battlefield or back in a
            // command zone. `find_card_anywhere` covers the former (+ other
            // zones) but deliberately skips the command zone, so fall back to
            // scanning each player's command zone for the name/owner.
            let source = state.find_card_anywhere(*source_id).or_else(|| {
                state
                    .players
                    .iter()
                    .find_map(|p| p.command.iter().find(|c| c.id == *source_id))
            });
            crate::net::CommanderDamageEntry {
                source_name: source
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_else(|| "Commander".to_string()),
                source_seat: source.map(|c| c.owner).unwrap_or(0),
                amount: *amount,
            }
        })
        .collect();
    // Closest-to-lethal first; tie-break on name for a stable order across
    // frames (HashMap iteration order is otherwise nondeterministic).
    entries.sort_by(|a, b| {
        b.amount
            .cmp(&a.amount)
            .then_with(|| a.source_name.cmp(&b.source_name))
    });
    entries
}

#[allow(clippy::too_many_arguments)]
fn project_player(
    player: &Player,
    player_seat: usize,
    viewer_seat: usize,
    prevention_shields: &[crate::game::types::PreventionShield],
    devotion: [u32; 5],
    draw_cap: Option<u32>,
    is_monarch: bool,
    commander_damage_taken: Vec<crate::net::CommanderDamageEntry>,
    team: usize,
) -> PlayerView {
    use crate::game::types::PreventionTarget;
    let has_prevention_shield = prevention_shields
        .iter()
        .any(|s| s.target == PreventionTarget::Player(player_seat));
    PlayerView {
        seat: player_seat,
        name: player.name.clone(),
        life: player.life,
        poison_counters: player.poison_counters,
        energy: player.energy,
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
        draw_cap,
        cards_left_graveyard_this_turn: player.cards_left_graveyard_this_turn,
        creatures_died_this_turn: player.creatures_died_this_turn,
        cards_exiled_this_turn: player.cards_exiled_this_turn,
        instants_or_sorceries_cast_this_turn: player.instants_or_sorceries_cast_this_turn,
        creatures_cast_this_turn: player.creatures_cast_this_turn,
        spells_cast_this_turn: player.spells_cast_this_turn,
        max_hand_size: player.max_hand_size,
        // Command zone is public — every viewer sees every card as
        // `Known`. We reuse `HandCardView` for the card shape since
        // it already carries name / cost / types / target hints,
        // which is what the UI needs to render and previs casting.
        command: player
            .command
            .iter()
            .map(|c| HandCardView::Known(known_card(c)))
            .collect(),
        commanders: player.commanders.clone(),
        eliminated: player.eliminated,
        emblems: player.emblems.iter().map(|e| e.name.clone()).collect(),
        has_prevention_shield,
        devotion,
        is_monarch,
        has_city_blessing: player.city_blessing,
        commander_damage_taken,
        team,
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
    let cycling_cost = card.definition.keywords.iter().find_map(|kw| {
        if let crate::card::Keyword::Cycling(c) = kw {
            Some(c.clone())
        } else {
            None
        }
    });
    let landcycling_cost = card.definition.keywords.iter().find_map(|kw| {
        if let crate::card::Keyword::Landcycling(c, _) = kw {
            Some(c.clone())
        } else {
            None
        }
    });
    let (modal_descriptions, modal_needs_target) =
        if let crate::effect::Effect::ChooseMode(modes) = &card.definition.effect {
            let descs = modes.iter().map(|m| m.effect_short_text()).collect();
            let needs = modes.iter().map(|m| m.requires_target()).collect();
            (descs, needs)
        } else {
            (Vec::new(), Vec::new())
        };
    KnownCard {
        id: card.id,
        name: card.definition.name.to_string(),
        cost: card.definition.cost.clone(),
        card_types: card.definition.card_types.clone(),
        needs_target: card.definition.effect.requires_target(),
        has_alternative_cost: card.definition.alternative_cost.is_some(),
        alt_cost_needs_pitch: card
            .definition
            .alternative_cost
            .as_ref()
            .is_some_and(|a| a.exile_filter.is_some()),
        alt_cost_label: card
            .definition
            .alternative_cost
            .as_ref()
            .map(|a| format_mana_cost_for_label(&a.mana_cost))
            .unwrap_or_default(),
        back_face_name: card
            .definition
            .back_face
            .as_ref()
            .map(|b| b.name.to_string()),
        has_cycling: cycling_cost.is_some(),
        cycling_cost_label: cycling_cost
            .as_ref()
            .map(format_mana_cost_for_label)
            .unwrap_or_default(),
        has_landcycling: landcycling_cost.is_some(),
        landcycling_cost_label: landcycling_cost
            .as_ref()
            .map(format_mana_cost_for_label)
            .unwrap_or_default(),
        modal_descriptions,
        modal_needs_target,
    }
}

/// Render a ManaCost as `{1}{U}` / `{R}{R}` / `{X}{X}` for client
/// labels. Mirrors how cost pips are rendered on Scryfall but with
/// the curly-brace symbology preserved (the client font handles the
/// rest). Pure helper — no game-state side effects.
/// Render a ManaCost as its `{2}{W}{B}` printed-Oracle representation.
/// Thin wrapper around `ManaCost::summary` — but special-cases the
/// empty-cost case to the empty string rather than `{0}` since the
/// server's existing callers (cycling cost label, etc.) prefer a
/// blank slot to a literal `{0}` when the cost is structurally absent.
fn format_mana_cost_for_label(c: &crate::mana::ManaCost) -> String {
    if c.symbols.is_empty() {
        return String::new();
    }
    c.summary()
}

fn graveyard_entry(card: &CardInstance) -> GraveyardCardView {
    GraveyardCardView {
        id: card.id,
        name: card.definition.name.to_string(),
        card_types: card.definition.card_types.clone(),
        mana_cost: card.definition.cost.clone(),
        power: card.definition.base_power(),
        toughness: card.definition.base_toughness(),
        flashback_cost: card.definition.has_flashback().cloned(),
        retrace: card.definition.has_retrace(),
        escape: card.definition.has_escape().map(|(c, n)| (c.clone(), n)),
        bestow_cost: card.definition.has_bestow().cloned(),
        buyback_cost: card.definition.has_buyback().cloned(),
    }
}

fn project_permanent(
    card: &CardInstance,
    computed: &[crate::game::layers::ComputedPermanent],
    attacking: &[CardId],
    block_map: &[(CardId, CardId)],
    prevention_shields: &[crate::game::types::PreventionShield],
    battlefield: &[CardInstance],
) -> PermanentView {
    use crate::game::types::PreventionTarget;
    let cp = computed.iter().find(|c| c.id == card.id);
    let has_prevention_shield = prevention_shields
        .iter()
        .any(|s| s.target == PreventionTarget::Permanent(card.id));
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
        blocking_attacker: block_map
            .iter()
            .find_map(|(b, a)| (*b == card.id).then_some(*a)),
        abilities: project_abilities(card),
        loyalty_abilities: project_loyalty_abilities(card),
        triggered_ability_labels: project_triggered_ability_labels(card),
        static_ability_labels: project_static_ability_labels(card),
        has_stun_counters: card.counter_count(crate::card::CounterType::Stun) > 0,
        has_finality_counters: card.counter_count(crate::card::CounterType::Finality) > 0,
        has_shield_counters: card.counter_count(crate::card::CounterType::Shield) > 0,
        has_prevention_shield,
        goaded: !card.goaded_by.is_empty(),
        monstrous: card.monstrous,
        suspected: card.suspected,
        pt_modified: {
            let cp_power = cp.map(|c| c.power).unwrap_or_else(|| card.power());
            let cp_toughness = cp.map(|c| c.toughness).unwrap_or_else(|| card.toughness());
            card.definition.is_creature()
                && (cp_power != card.definition.base_power()
                    || cp_toughness != card.definition.base_toughness())
        },
        mana_cost_display: format_mana_cost(&card.definition.cost),
        creature_types: card
            .definition
            .subtypes
            .creature_types
            .iter()
            .map(|ct| format!("{ct:?}"))
            .collect(),
        // Generic-mana Ward cost surfaced for client UI. Non-mana Ward
        // variants (Life / Discard / Sacrifice) fall through as 0 — a
        // future field can carry the richer WardCost shape if a client
        // needs it.
        ward_cost: card.definition.keywords.iter().find_map(|kw| {
            if let crate::card::Keyword::Ward(crate::card::WardCost::Mana(c)) = kw {
                Some(c.cmc())
            } else {
                None
            }
        }).unwrap_or(0),
        mana_value: card.definition.cost.cmc(),
        is_legendary: card.definition.supertypes.contains(&crate::card::Supertype::Legendary),
        has_plus_one_counters: card.counter_count(crate::card::CounterType::PlusOnePlusOne) > 0,
        has_minus_one_counters: card.counter_count(crate::card::CounterType::MinusOneMinusOne) > 0,
        total_counter_count: card.counters.values().sum(),
        keyword_counters: card.keyword_counters
            .iter()
            .filter(|(_, n)| **n > 0)
            .map(|(k, n)| (k.clone(), *n))
            .collect(),
        shield_counter_count: card.counter_count(crate::card::CounterType::Shield),
        stun_counter_count: card.counter_count(crate::card::CounterType::Stun),
        finality_counter_count: card.counter_count(crate::card::CounterType::Finality),
        regeneration_shields: card.regeneration_shields,
        equippable: card.definition.is_equipment() && card.definition.has_equip().is_some(),
        crew_value: card.definition.crew_cost().unwrap_or(0),
        marked_lethal: {
            let tough = cp.map(|c| c.toughness).unwrap_or_else(|| card.toughness());
            let indestructible = cp
                .map(|c| c.keywords.contains(&crate::card::Keyword::Indestructible))
                .unwrap_or_else(|| card.has_keyword(&crate::card::Keyword::Indestructible));
            card.definition.is_creature()
                && !indestructible
                && tough > 0
                && card.damage as i32 >= tough
        },
        named_card: card.named_card.clone(),
        chosen_color: card.chosen_color,
        // Auras / Equipment / Fortifications attached to this permanent.
        attachments: battlefield
            .iter()
            .filter(|o| o.attached_to == Some(card.id))
            .map(|o| o.definition.name.to_string())
            .collect(),
        // CR 702.95 — Soulbond partner (only while still on the battlefield).
        soulbond_partner: card
            .soulbond_partner
            .filter(|p| battlefield.iter().any(|o| o.id == *p)),
    }
}

/// Project the printed `StaticAbility.description` strings as a flat
/// `Vec<String>` for the client tooltip. Cards without static
/// abilities yield an empty vector. The descriptions are 'static and
/// stable across recomputes — they're the printed Oracle wording.
fn project_static_ability_labels(card: &CardInstance) -> Vec<String> {
    card.definition
        .static_abilities
        .iter()
        .map(|s| s.description.to_string())
        .filter(|d| !d.is_empty())
        .collect()
}

/// Generate one-line summaries per triggered ability for the client
/// tooltip. Format: "Event: Effect" e.g. "ETB: Draw a card",
/// "Magecraft: Drain 1", "Dies: Mill 2". The trigger-event prefix is
/// inferred from the `EventSpec.kind` + `EventScope` pair via
/// `trigger_event_label`; the effect body uses the existing
/// `ability_effect_label`.
fn project_triggered_ability_labels(card: &CardInstance) -> Vec<String> {
    // Printed triggers plus any an Equipment grants to the creature it's
    // attached to (CR 702.6e — `EquipBonus.triggered_abilities`, the Sword
    // cycle's combat-damage riders), so the tooltip shows the full set.
    let granted = card
        .definition
        .equipped_bonus
        .as_ref()
        .map(|b| b.triggered_abilities.as_slice())
        .unwrap_or(&[]);
    card.definition
        .triggered_abilities
        .iter()
        .chain(granted)
        .map(|t| {
            let evt = trigger_event_label(&t.event);
            let eff = ability_effect_label(&t.effect);
            if evt.is_empty() {
                eff.to_string()
            } else {
                format!("{evt}: {eff}")
            }
        })
        .collect()
}

/// Short human label for a trigger event-spec. Used as the prefix in
/// `project_triggered_ability_labels`. Returns an empty string for
/// unrecognized event kinds so the caller can fall back to the bare
/// effect label.
fn trigger_event_label(event: &crate::card::EventSpec) -> &'static str {
    use crate::card::{EventKind, EventScope, Predicate};
    // Magecraft pattern: SpellCast / YourControl with the IS-filter.
    if matches!(event.kind, EventKind::SpellCast)
        && matches!(event.scope, EventScope::YourControl)
    {
        // Check the filter for the canonical "instant or sorcery"
        // gate — that's a magecraft trigger.
        let is_magecraft = matches!(
            &event.filter,
            Some(Predicate::All(parts))
                if parts.iter().any(|p| matches!(
                    p, Predicate::EntityMatches { .. }
                ))
        ) || matches!(
            &event.filter,
            Some(Predicate::EntityMatches { .. })
        );
        if is_magecraft {
            return "Magecraft";
        }
        return "Spell cast";
    }
    match (&event.kind, event.scope) {
        (EventKind::EntersBattlefield, EventScope::SelfSource) => "ETB",
        (EventKind::EntersBattlefield, EventScope::AnotherOfYours) => "Another ETB",
        (EventKind::EntersBattlefield, EventScope::AnyPlayer) => "Any ETB",
        (EventKind::CreatureDied, EventScope::SelfSource) => "Dies",
        (EventKind::CreatureDied, EventScope::AnotherOfYours) => "Other dies",
        (EventKind::CreatureDied, EventScope::AnyPlayer) => "Creature dies",
        (EventKind::CreatureSacrificed, EventScope::SelfSource) => "Sacrificed",
        (EventKind::CreatureSacrificed, EventScope::YourControl) => "You sacrifice",
        (EventKind::PermanentSacrificed, EventScope::YourControl) => "You sacrifice",
        (EventKind::PermanentLeavesBattlefield, _) => "Leaves bf",
        (EventKind::Attacks, EventScope::SelfSource) => "Attacks",
        (EventKind::Attacks, EventScope::YourControl) => "You attack",
        (EventKind::Attacks, EventScope::AnotherOfYours) => "Another attacks",
        (EventKind::Blocks, EventScope::SelfSource) => "Blocks",
        (EventKind::BecomesBlocked, EventScope::SelfSource) => "Becomes blocked",
        (EventKind::AttacksAndIsntBlocked, EventScope::SelfSource) => "Unblocked",
        (EventKind::CardCycled, EventScope::SelfSource) => "Cycle",
        (EventKind::CardCycled, EventScope::YourControl) => "You cycle",
        (EventKind::CardDrawn, EventScope::YourControl) => "On draw",
        (EventKind::CardDrawn, EventScope::SelfSource) => "On self-draw",
        (EventKind::CardDiscarded, EventScope::YourControl) => "On discard",
        (EventKind::LifeGained, EventScope::YourControl) => "On lifegain",
        (EventKind::LifeGained, EventScope::AnyPlayer) => "Any lifegain",
        (EventKind::LifeLost, EventScope::YourControl) => "On life loss",
        (EventKind::LifeLost, EventScope::OpponentControl) => "Opp life loss",
        (EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource) => "Combat dmg",
        (EventKind::DealsCombatDamageToPlayer, EventScope::YourControl) => "Your combat dmg",
        (EventKind::DealsCombatDamageToCreature, EventScope::SelfSource) => "Combat dmg to crea",
        (EventKind::LandPlayed, EventScope::YourControl) => "Landfall",
        (EventKind::LandPlayed, EventScope::AnyPlayer) => "Any landfall",
        (EventKind::SpellCast, EventScope::OpponentControl) => "Opp casts",
        (EventKind::SpellCast, EventScope::AnyPlayer) => "Any cast",
        (EventKind::TurnBegins, _) => "Turn begins",
        (EventKind::CardLeftGraveyard, EventScope::YourControl) => "GY leaves",
        (EventKind::CardLeftGraveyard, EventScope::AnyPlayer) => "Any GY leave",
        (EventKind::CounterAdded(_), EventScope::SelfSource) => "On counter",
        (EventKind::CounterAdded(_), EventScope::YourControl) => "On any counter",
        (EventKind::AbilityActivated, _) => "Ability activated",
        (EventKind::BecameTarget, EventScope::SelfSource) => "Becomes target",
        (EventKind::Blocks, EventScope::AnotherOfYours) => "Another blocks",
        (EventKind::BecomesBlocked, EventScope::YourControl) => "Your blocker",
        (EventKind::PermanentSacrificed, EventScope::SelfSource) => "Self sac",
        (EventKind::PermanentSacrificed, EventScope::AnyPlayer) => "Any sac",
        (EventKind::CreatureSacrificed, EventScope::AnyPlayer) => "Any creature sac",
        (EventKind::CardDrawn, EventScope::AnyPlayer) => "Any draw",
        (EventKind::CardDiscarded, EventScope::AnyPlayer) => "Any discard",
        (EventKind::CardDiscarded, EventScope::SelfSource) => "Self discard",
        (EventKind::LifeGained, EventScope::SelfSource) => "Self gains life",
        (EventKind::LifeLost, EventScope::SelfSource) => "Self loses life",
        (EventKind::LifeLost, EventScope::AnyPlayer) => "Any life loss",
        (EventKind::Attacks, EventScope::AnyPlayer) => "Any attacks",
        (EventKind::Attacks, EventScope::OpponentControl) => "Opp attacks",
        (EventKind::AttacksAndIsntBlocked, EventScope::YourControl) => "Your unblocked",
        (EventKind::StepBegins(crate::game::types::TurnStep::Untap), _) => "Untap step",
        (EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), _) => "Upkeep",
        (EventKind::StepBegins(crate::game::types::TurnStep::Draw), _) => "Draw step",
        (EventKind::StepBegins(crate::game::types::TurnStep::PreCombatMain), _) => "Main 1",
        (EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat), _) => "Begin combat",
        (EventKind::StepBegins(crate::game::types::TurnStep::PostCombatMain), _) => "Main 2",
        (EventKind::StepBegins(crate::game::types::TurnStep::End), _) => "End step",
        (EventKind::StepBegins(_), _) => "Step",
        (EventKind::SpellCast, EventScope::SelfSource) => "On cast",
        (EventKind::LandPlayed, EventScope::FromYourGraveyard) => "Landfall (gy)",
        (EventKind::LandPlayed, EventScope::OpponentControl) => "Opp landfall",
        (EventKind::CreatureDied, EventScope::OpponentControl) => "Opp creature dies",
        (EventKind::EntersBattlefield, EventScope::YourControl) => "Your ETB",
        (EventKind::EntersBattlefield, EventScope::OpponentControl) => "Opp ETB",
        // Trigger labels added in batch 167 — fills remaining coverage
        // gaps in the dispatcher matrix. Each one corresponds to an
        // EventKind × EventScope pair that previously fell into the
        // `""` catch-all and would render as an empty tooltip on the
        // client trigger panel.
        (EventKind::Blocks, EventScope::AnyPlayer) => "Any blocks",
        (EventKind::Blocks, EventScope::YourControl) => "You block",
        (EventKind::Blocks, EventScope::OpponentControl) => "Opp blocks",
        (EventKind::BecomesBlocked, EventScope::OpponentControl) => "Opp blocked",
        (EventKind::BecomesBlocked, EventScope::AnyPlayer) => "Any blocked",
        (EventKind::DealsCombatDamageToPlayer, EventScope::OpponentControl) => "Opp combat dmg",
        (EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer) => "Any combat dmg",
        (EventKind::DealsCombatDamageToCreature, EventScope::YourControl) => "Your combat dmg crea",
        (EventKind::DealsCombatDamageToCreature, EventScope::AnyPlayer) => "Any combat dmg crea",
        (EventKind::CardCycled, EventScope::AnyPlayer) => "Any cycle",
        (EventKind::CardCycled, EventScope::OpponentControl) => "Opp cycle",
        (EventKind::CardLeftGraveyard, EventScope::SelfSource) => "Self GY leave",
        (EventKind::CardLeftGraveyard, EventScope::OpponentControl) => "Opp GY leave",
        (EventKind::CounterAdded(_), EventScope::AnyPlayer) => "Any counter",
        (EventKind::CounterAdded(_), EventScope::OpponentControl) => "Opp counter",
        (EventKind::BecameTarget, EventScope::YourControl) => "You target",
        (EventKind::BecameTarget, EventScope::OpponentControl) => "Opp targets",
        (EventKind::BecameTarget, EventScope::AnyPlayer) => "Any targets",
        (EventKind::CreatureSacrificed, EventScope::OpponentControl) => "Opp creature sac",
        (EventKind::PermanentSacrificed, EventScope::OpponentControl) => "Opp permanent sac",
        // Enrage (CR 702.130) — "Whenever this creature is dealt damage."
        (EventKind::DealtDamage, EventScope::SelfSource) => "Enrage",
        (EventKind::DealtDamage, EventScope::YourControl) => "Your crea dealt dmg",
        (EventKind::DealtDamage, EventScope::AnyPlayer) => "Any crea dealt dmg",
        // Scope-aware fallback for any EventKind x EventScope pair not
        // enumerated above. Previously these fell through to "" and
        // rendered as a blank trigger chip on the client; a scope-tagged
        // generic ("Triggered" / "Your trigger" / "Opp trigger") is
        // always non-empty so the tooltip is never blank.
        (_, EventScope::OpponentControl) => "Opp trigger",
        (_, EventScope::YourControl | EventScope::AnotherOfYours) => "Your trigger",
        (_, _) => "Triggered",
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
        Predicate::LifeGainedThisTurnAtLeast { at_least: Value::Const(1), .. } => {
            "after lifegain".into()
        }
        Predicate::CardsExiledThisTurnAtLeast {
            at_least: Value::Const(1), ..
        } => "after exile".into(),
        Predicate::CreaturesDiedThisTurnAtLeast {
            at_least: Value::Const(1), ..
        } => "after creature death".into(),
        Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) } => {
            "morbid".into()
        }
        Predicate::CastSpellHasX => "cast spell w/ {X}".into(),
        Predicate::CastSpellTargetsMatch(_) => "cast spell targets match".into(),
        Predicate::CastSpellManaSpentAtLeast(n) => format!("if ≥{n} mana spent"),
        Predicate::IncrementSatisfied => "Increment (mana > P or T)".into(),
        // Catch-all: no human-readable form yet.
        _ => "conditional".into(),
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
            ManaSymbol::MonoHybrid(n, c) => format!("{{{n}/{c}}}"),
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
    // Graveyard-source activations (push XVII): "Exile this from gy"
    // covers the exile-self-as-cost variant (Stone Docent, Eternal
    // Student). Plain `from_graveyard: true` without exile_self_cost
    // (Summoned Dromedary, Teacher's Pest) is rendered through the
    // effect label — the source's return-to-hand or return-to-bf
    // effect already signals "from gy".
    if ability.exile_self_cost {
        parts.push("Exile this from gy".into());
    }
    // Lorehold Pledgemage / Postmortem Professor — "Exile a card from
    // your graveyard" as an additional cost (count 1). Grim Lavamancer —
    // "Exile two cards from your graveyard" (count 2). The bare label
    // pluralises off the exile count.
    if let Some((_, n)) = ability.exile_other_filter.as_ref() {
        if *n == 1 {
            parts.push("Exile a card from gy".into());
        } else {
            parts.push(format!("Exile {n} cards from gy"));
        }
    }
    // Sacrifice-another-permanent activations (Phyrexian Tower, Witherbloom
    // sac-outlets, Carrion Feeder, etc.) — `sac_other_filter: Some((req, n))`.
    // Without this the ability looks free for its tap+mana alone.
    if let Some((req, n)) = ability.sac_other_filter.as_ref() {
        let noun = requirement_noun(req);
        if *n == 1 {
            parts.push(format!("Sacrifice a {noun}"));
        } else {
            parts.push(format!("Sacrifice {n} {noun}s"));
        }
    }
    // Tap-another-creature activations (Convoke-style outlets, "Tap an
    // untapped creature you control" costs) — `tap_other_filter`.
    if let Some(req) = ability.tap_other_filter.as_ref() {
        parts.push(format!("Tap a {}", requirement_noun(req)));
    }
    if parts.is_empty() { "0".into() } else { parts.join(", ") }
}

/// Short noun for the common `SelectionRequirement` shapes used in cost
/// riders ("Sacrifice a [noun]"). Falls back to "permanent" for filters
/// without a crisp single-word label.
fn requirement_noun(req: &crate::card::SelectionRequirement) -> &'static str {
    use crate::card::SelectionRequirement as R;
    match req {
        R::Creature => "creature",
        R::Artifact => "artifact",
        R::Enchantment => "enchantment",
        R::Land => "land",
        R::Planeswalker => "planeswalker",
        // Peel a leading And to read the primary type (e.g. Creature ∧
        // ControlledByYou → "creature").
        R::And(a, _) => requirement_noun(a),
        _ => "permanent",
    }
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
        Effect::ChooseN { modes, .. } => modes
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
        Effect::DoubleLife { .. } => "Double life total",
        Effect::ShuffleSelfIntoLibrary => "Shuffle into library",
        Effect::Mill { .. } => "Mill",
        Effect::Scry { .. } => "Scry",
        Effect::Surveil { .. } => "Surveil",
        Effect::AddCounter { .. } => "Add counter",
        Effect::RemoveCounter { .. } => "Remove counter",
        Effect::CreateToken { .. } => "Create token",
        Effect::CounterSpell { .. } => "Counter spell",
        Effect::CounterSpellToZone { .. } => "Counter spell (alt zone)",
        Effect::CounterAbility { .. } => "Counter ability",
        Effect::CounterUnlessPaid { .. } => "Counter unless paid",
        Effect::CounterUnless { .. } => "Ward (counter unless cost paid)",
        Effect::Sacrifice { .. } | Effect::SacrificeAndRemember { .. } => "Sacrifice",
        Effect::SacrificeAnyNumber { .. } => "Sacrifice any number",
        Effect::PayLifeLookTake { .. } => "Pay life, dig, take one",
        Effect::DiscardChosen { .. } => "Discard chosen",
        Effect::ExileChosenFromHand { .. } => "Exile chosen from hand",
        Effect::PayOrLoseGame { .. } => "Pay or lose",
        Effect::DelayUntil { .. } => "Delayed trigger",
        Effect::Tap { .. } => "Tap",
        Effect::Untap { .. } => "Untap",
        Effect::PumpPT { .. } => "Pump",
        Effect::SetBasePT { .. } => "Set base P/T",
        Effect::Process { then, .. } => {
            // Surface the rider's label — the "process from exile" step
            // resolves through the decision panel.
            let inner = ability_effect_label(then);
            if inner == "Activate" { "Process" } else { inner }
        }
        Effect::GrantKeyword { .. } => "Grant keyword",
        Effect::AddPoison { .. } => "Add poison",
        Effect::RevealUntilFind { .. } => "Reveal until find",
        Effect::AddFirstSpellTax { .. } => "Cost tax",
        Effect::Drain { .. } => "Drain",
        Effect::SetNoMaxHandSize { .. } => "No max hand size",
        Effect::FlipCoin { .. } => "Flip coin",
        Effect::Proliferate => "Proliferate",
        Effect::LookAtTop { .. } => "Look at top",
        Effect::RearrangeTop { .. } => "Rearrange top",
        Effect::ShuffleGraveyardIntoLibrary { .. } => "Shuffle into library",
        Effect::PutOnLibraryFromHand { .. } => "Put on library",
        Effect::RevealTopAndDrawIf { .. } => "Reveal top",
        Effect::CopySpell { .. } => "Copy spell",
        Effect::CopySpellMayChooseTargets { .. } => "Copy spell (new targets)",
        Effect::ChooseNewTargetsForSpell { .. } => "Choose new targets",
        Effect::GainControl { .. } => "Gain control",
        Effect::ResetCreature { .. } => "Reset creature",
        Effect::BecomeBasicLand { .. } => "Become basic land",
        Effect::Attach { .. } => "Attach",
        Effect::GrantSorceriesAsFlash { .. } => "Sorceries as flash",
        Effect::NameCreatureType { .. } => "Name creature type",
        Effect::GrantTriggeredAbility { .. } => "Grant ability",
        Effect::LoseAllAbilities { .. } => "Remove abilities",
        Effect::DiscardAnyNumber { .. } => "Discard any number",
        Effect::SacrificeGreatestMV { .. } => "Sacrifice (highest MV)",
        Effect::CopySpellUnlessPaid { .. } => "Copy unless paid",
        Effect::GrantMayPlay { .. } => "Grant may play",
        Effect::CastWithoutPayingImmediate { .. } => "Cast free",
        Effect::RegisterParadigm => "Paradigm",
        Effect::CastFreeParadigmCopy => "Cast paradigm copy",
        Effect::WinGame { .. } => "Win the game",
        Effect::PreventAllCombatDamageThisTurn => "Prevent combat damage",
        Effect::PreventAllCombatDamageInvolving { .. } => "Prevent combat damage to/from target",
        Effect::CantBlockSourceThisTurn { .. } => "Target can't block this",
        Effect::SkipTurns { .. } => "Skip turns",
        Effect::SetLifeTotal { .. } => "Set life total",
        Effect::ExchangeLifeTotals { .. } => "Exchange life totals",
        Effect::PreventNextDamage { .. } => "Prevent damage",
        Effect::PreventNextDamageAndGainLife { .. } => "Prevent damage, gain life",
        Effect::PreventAllDamageThisTurn { .. } => "Prevent all damage",
        Effect::DamageCantBePreventedThisTurn => "Damage can't be prevented",
        Effect::LifeGainLockThisTurn { .. } => "Lock lifegain",
        Effect::GrantSpellsUncounterableThisTurn { .. } => "Spells can't be countered",
        Effect::Explore { .. } => "Explore",
        Effect::Goad { .. } => "Goad",
        Effect::Provoke { .. } => "Provoke",
        Effect::Monstrosity { .. } => "Monstrosity",
        Effect::MoveCounter { .. } => "Move counters",
        Effect::RevealTopCard { .. } => "Reveal top card",
        Effect::RevealTopLandToBattlefieldElseHand { .. } => "Reveal top; land to play else hand",
        Effect::ManaClash { .. } => "Mana Clash (flip-off)",
        Effect::RollDie { .. } => "Roll die",
        Effect::IfRevealFromHand { .. } => "Reveal from hand",
        Effect::DiminishCreaturesExceptChosenType { .. } => "Diminish creatures",
        Effect::CreateTokenCopyOf { .. } => "Copy permanent",
        Effect::CreateEmblem { .. } => "Get an emblem",
        Effect::TakeExtraTurn { .. } => "Take an extra turn",
        Effect::ExileAnyNumberFromGraveyards { .. } => "Exile cards from graveyards",
        Effect::ExileAllGraveyards => "Exile all graveyards",
        Effect::CreateTokenAttacking { .. } => "Create attacking tokens",
        Effect::Amass { .. } => "Amass",
        Effect::Myriad => "Myriad",
        Effect::Enlist => "Enlist",
        Effect::GrantNextInstantOrSorceryDiscountThisTurn { .. } => "Discount next spell",
        Effect::SupportCounters { .. } => "Support",
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
        StackItem::Spell { card, caster, target, additional_targets, .. } => {
            StackItemView::Known(KnownStackItem {
                source: card.id,
                controller: *caster,
                name: card.definition.name.to_string(),
                target: target.clone(),
                additional_targets: additional_targets.clone(),
                kind: StackItemKind::Spell,
            })
        }
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
                additional_targets: vec![],
                kind: StackItemKind::Trigger,
            })
        }
    }
}

fn format_mana_cost(cost: &crate::mana::ManaCost) -> String {
    // Delegate to the canonical `ManaCost::summary()` renderer so every
    // pip kind (colored, generic, {C} colorless, {S} snow, {X}, hybrid,
    // Phyrexian, mono-hybrid) is rendered with the proper Oracle-style
    // letters. The previous hand-rolled match used Debug formatting for
    // hybrid / Phyrexian colors (`{White/Black}` instead of `{W/B}`) and
    // mis-rendered {C} pips as generic. An empty cost stays the empty
    // string here (summary() renders "{0}" for free spells; the card
    // view wants nothing displayed for a 0-symbol cost).
    if cost.symbols.is_empty() {
        return String::new();
    }
    cost.summary()
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
    fn project_surfaces_gravestorm_count() {
        // CR 702.69 — the view exposes the turn's permanents-to-graveyard
        // tally so the client can badge a Gravestorm count.
        let mut g = two_player_game();
        let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_from_battlefield_to_graveyard(a);
        g.remove_from_battlefield_to_graveyard(b);
        let view = project(&g, 0);
        assert_eq!(view.permanents_to_graveyard_this_turn, 2);
    }

    #[test]
    fn format_mana_cost_renders_pip_letters_not_debug_names() {
        use crate::mana::{cost, generic, hybrid, mono_hybrid, phyrexian, w, b, Color, ManaCost};
        // Two-color hybrid renders as {W/B}, not the Debug {White/Black}.
        assert_eq!(
            format_mana_cost(&cost(&[generic(1), hybrid(Color::White, Color::Black)])),
            "{1}{W/B}",
        );
        // Phyrexian renders {B/P}.
        assert_eq!(format_mana_cost(&cost(&[phyrexian(Color::Black)])), "{B/P}");
        // Mono-hybrid renders {2/R}.
        assert_eq!(format_mana_cost(&cost(&[mono_hybrid(2, Color::Red)])), "{2/R}");
        // Plain colored + generic.
        assert_eq!(format_mana_cost(&cost(&[generic(2), w(), b()])), "{2}{W}{B}");
        // Empty cost stays the empty string (not "{0}").
        assert_eq!(format_mana_cost(&ManaCost::new(vec![])), "");
    }

    #[test]
    fn prevention_shields_surface_in_the_view() {
        use crate::game::types::{PreventionShield, PreventionTarget};
        let mut state = two_player_game();
        let bear = state.add_card_to_battlefield(1, catalog::grizzly_bears());
        state.prevention_shields.push(PreventionShield {
            target: PreventionTarget::Player(0),
            remaining: None,
            gain_life: false,
        });
        state.prevention_shields.push(PreventionShield {
            target: PreventionTarget::Permanent(bear),
            remaining: Some(2),
            gain_life: false,
        });
        state.damage_cant_be_prevented_this_turn = true;
        let v = project(&state, 0);
        assert!(v.players[0].has_prevention_shield, "P0 is shielded");
        assert!(!v.players[1].has_prevention_shield, "P1 is not");
        assert!(v.battlefield.iter().find(|p| p.id == bear).unwrap().has_prevention_shield);
        assert!(v.damage_cant_be_prevented_this_turn);
    }

    #[test]
    fn goaded_and_monstrous_surface_in_the_view() {
        let mut state = two_player_game();
        let a = state.add_card_to_battlefield(1, catalog::grizzly_bears());
        let b = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        state.battlefield_find_mut(a).unwrap().goaded_by = vec![0];
        state.battlefield_find_mut(b).unwrap().monstrous = true;
        let v = project(&state, 0);
        assert!(v.battlefield.iter().find(|p| p.id == a).unwrap().goaded);
        assert!(v.battlefield.iter().find(|p| p.id == b).unwrap().monstrous);
        assert!(!v.battlefield.iter().find(|p| p.id == a).unwrap().monstrous);
    }

    #[test]
    fn suspected_surfaces_in_the_view() {
        let mut state = two_player_game();
        let a = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        state.battlefield_find_mut(a).unwrap().suspected = true;
        let v = project(&state, 0);
        assert!(v.battlefield.iter().find(|p| p.id == a).unwrap().suspected);
        assert!(!v.battlefield.iter().find(|p| p.id == b).unwrap().suspected);
    }

    #[test]
    fn devotion_surfaces_per_color_in_the_view() {
        let mut state = two_player_game();
        // P0: Erebos ({3}{B}) + Gray Merchant ({3}{B}{B}) → 3 black pips.
        state.add_card_to_battlefield(0, catalog::erebos_god_of_the_dead());
        state.add_card_to_battlefield(0, catalog::gray_merchant_of_asphodel());
        let v = project(&state, 0);
        // Index 2 = Black (W,U,B,R,G).
        assert_eq!(v.players[0].devotion[2], 3, "devotion to black = 3");
        assert_eq!(v.players[0].devotion[0], 0, "no white devotion");
        assert_eq!(v.players[1].devotion[2], 0, "opponent has no devotion");
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
            additional_targets: vec![],
            mode: None,
            x_value: 0,
            converged_value: 0,
            mana_spent: 0,
            uncounterable: false,
        });
        g.stack.push(StackItem::Trigger {
            source: bolt_id,
            controller: 0,
            effect: Box::new(Effect::Noop),
            target: None,
            mode: None,
            x_value: 0,
            converged_value: 0,
        trigger_source: None,
            mana_spent: 0,
            event_amount: 0,
            intervening_if: None,
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
    fn stack_view_surfaces_all_targets_for_multi_target_spell() {
        use crate::game::StackItem;
        use crate::game::types::Target;
        let mut g = two_player_game();
        let bolt_id = g.add_card_to_battlefield(0, catalog::lightning_bolt());
        let bolt = g.battlefield_find(bolt_id).cloned().unwrap();
        g.battlefield.retain(|c| c.id != bolt_id);
        g.stack.push(StackItem::Spell {
            card: Box::new(bolt),
            caster: 0,
            target: Some(Target::Player(1)),
            additional_targets: vec![Target::Player(0)],
            mode: None,
            x_value: 0,
            converged_value: 0,
            mana_spent: 0,
            uncounterable: false,
        });
        let v = project(&g, 0);
        match &v.stack[0] {
            StackItemView::Known(k) => {
                assert_eq!(k.target, Some(Target::Player(1)));
                assert_eq!(k.additional_targets, vec![Target::Player(0)],
                    "view must surface slots 1+ so the UI can arrow every target");
            }
            _ => panic!("expected Known"),
        }
    }

    #[test]
    fn marked_lethal_flags_doomed_creatures_in_view() {
        let mut state = two_player_game();
        let bear = state.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
        // No damage → not lethal.
        assert!(!project(&state, 0).battlefield.iter()
            .find(|p| p.id == bear).unwrap().marked_lethal);
        // 2 damage on a 2-toughness creature → marked lethal.
        state.battlefield.iter_mut().find(|c| c.id == bear).unwrap().damage = 2;
        assert!(project(&state, 0).battlefield.iter()
            .find(|p| p.id == bear).unwrap().marked_lethal);
    }

    #[test]
    fn named_card_is_surfaced_in_permanent_view() {
        let mut state = two_player_game();
        let needle = state.add_card_to_battlefield(0, catalog::pithing_needle());
        assert_eq!(project(&state, 0).battlefield.iter()
            .find(|p| p.id == needle).unwrap().named_card, None);
        state.battlefield.iter_mut().find(|c| c.id == needle).unwrap()
            .named_card = Some("Tormod's Crypt".into());
        assert_eq!(project(&state, 0).battlefield.iter()
            .find(|p| p.id == needle).unwrap().named_card.as_deref(),
            Some("Tormod's Crypt"));
    }

    #[test]
    fn chosen_color_is_surfaced_in_permanent_view() {
        let mut state = two_player_game();
        let heart = state.add_card_to_battlefield(0, catalog::coldsteel_heart());
        assert_eq!(project(&state, 0).battlefield.iter()
            .find(|p| p.id == heart).unwrap().chosen_color, None);
        state.battlefield_find_mut(heart).unwrap().chosen_color = Some(crate::mana::Color::Blue);
        assert_eq!(project(&state, 0).battlefield.iter()
            .find(|p| p.id == heart).unwrap().chosen_color, Some(crate::mana::Color::Blue));
    }

    #[test]
    fn soulbond_partner_is_surfaced_in_permanent_view() {
        let mut state = two_player_game();
        let a = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = state.add_card_to_battlefield(0, catalog::wolfir_silverheart());
        assert_eq!(project(&state, 0).battlefield.iter()
            .find(|p| p.id == a).unwrap().soulbond_partner, None);
        state.battlefield_find_mut(a).unwrap().soulbond_partner = Some(b);
        state.battlefield_find_mut(b).unwrap().soulbond_partner = Some(a);
        assert_eq!(project(&state, 0).battlefield.iter()
            .find(|p| p.id == a).unwrap().soulbond_partner, Some(b));
        // A stale link to an off-battlefield card is suppressed.
        state.remove_from_battlefield_to_graveyard(b);
        assert_eq!(project(&state, 0).battlefield.iter()
            .find(|p| p.id == a).unwrap().soulbond_partner, None);
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
    fn trigger_event_label_is_never_blank() {
        use crate::card::{EventKind, EventScope, EventSpec};
        // Every EventKind x EventScope pair must produce a non-empty
        // label so the client never renders a blank trigger chip. This
        // covers pairs (e.g. LifeGained/OpponentControl,
        // DealtDamage/OpponentControl) that previously fell through to
        // the "" catch-all.
        let kinds = [
            EventKind::EntersBattlefield,
            EventKind::CreatureDied,
            EventKind::LifeGained,
            EventKind::LifeLost,
            EventKind::DealtDamage,
            EventKind::CardDrawn,
            EventKind::SpellCast,
            EventKind::Attacks,
        ];
        let scopes = [
            EventScope::SelfSource,
            EventScope::YourControl,
            EventScope::OpponentControl,
            EventScope::AnotherOfYours,
            EventScope::AnyPlayer,
        ];
        for k in &kinds {
            for s in &scopes {
                let spec = EventSpec::new(k.clone(), *s);
                let label = trigger_event_label(&spec);
                assert!(!label.is_empty(),
                    "label for {:?}/{:?} must not be blank", k, s);
            }
        }
    }

    #[test]
    fn trigger_event_label_fallback_is_scope_aware() {
        use crate::card::{EventKind, EventScope, EventSpec};
        // A pair with no explicit arm uses the scope-aware fallback.
        let opp = EventSpec::new(EventKind::LifeGained, EventScope::OpponentControl);
        assert_eq!(trigger_event_label(&opp), "Opp trigger");
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
    fn graveyard_view_surfaces_recast_options() {
        let mut state = two_player_game();
        // Raven's Crime carries Retrace; the view should advertise it.
        let crime = state.add_card_to_graveyard(0, catalog::ravens_crime());
        let view = project(&state, 0);
        let entry = view.players[0].graveyard.iter().find(|c| c.id == crime).unwrap();
        assert!(entry.retrace, "Retrace flagged on graveyard view");
        assert!(entry.flashback_cost.is_none(), "no flashback cost for Raven's Crime");
    }

    #[test]
    fn permanent_view_includes_static_ability_labels() {
        // Tenured Inkcaster has a printed static "Other Inkling
        // creatures you control get +2/+2." — the view should surface
        // that description string in `static_ability_labels`.
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::tenured_inkcaster());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert!(!perm.static_ability_labels.is_empty(),
            "Tenured Inkcaster has a printed static — view must surface it");
        assert!(
            perm.static_ability_labels.iter().any(|s| s.contains("Inkling")),
            "static_ability_labels should mention Inkling: {:?}",
            perm.static_ability_labels,
        );
    }

    #[test]
    fn permanent_view_surfaces_equipment_granted_triggers() {
        // Sword of Body and Mind grants a combat-damage trigger via
        // EquipBonus.triggered_abilities — the view must surface it.
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::sword_of_body_and_mind());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert!(perm.triggered_ability_labels.iter().any(|s| s.starts_with("Combat dmg")),
            "equipment-granted combat trigger should appear: {:?}",
            perm.triggered_ability_labels);
    }

    #[test]
    fn permanent_view_static_ability_labels_empty_for_vanilla_creature() {
        // Grizzly Bears has no static abilities — the view's
        // static_ability_labels should be empty.
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert!(perm.static_ability_labels.is_empty(),
            "vanilla creature has no statics");
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

    #[test]
    fn life_exchange_and_prevention_effects_have_labels() {
        // Magus of the Mirror's activated ability should label as the
        // exchange, not the generic "Activate" fallback.
        let magus = catalog::magus_of_the_mirror();
        assert_eq!(
            ability_effect_label(&magus.activated_abilities[0].effect),
            "Exchange life totals",
        );
        // Mending Hands's prevention effect.
        let mh = catalog::mending_hands();
        assert_eq!(ability_effect_label(&mh.effect), "Prevent damage");
    }

    #[test]
    fn amass_and_myriad_effects_have_labels() {
        use crate::effect::{PlayerRef, Value};
        let amass = Effect::Amass { who: PlayerRef::You, count: Value::Const(2), extra_type: None };
        assert_eq!(ability_effect_label(&amass), "Amass");
        assert_eq!(ability_effect_label(&Effect::Myriad), "Myriad");
    }

    #[test]
    fn devious_cover_up_exile_rider_has_graveyard_label() {
        use crate::card::SelectionRequirement;
        let eff = Effect::ExileAnyNumberFromGraveyards {
            filter: SelectionRequirement::Any,
        };
        assert_eq!(ability_effect_label(&eff), "Exile cards from graveyards");
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
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(2), w(), u(), b(), r()]),
            effect: Effect::Noop,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
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
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[x()]),
            effect: Effect::Noop,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
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
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Noop,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        };
        let label = ability_cost_label(&ab);
        assert!(label.contains("{1}"), "{label} must include the {{1}} cost");
        assert!(label.contains("{T}"), "{label} must include the tap cost");
        assert!(label.contains("Sac"),
            "{label} should advertise the sacrifice cost");

        // Lotus Petal: {T}, sac → add any one color. No mana cost.
        let petal = ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[]),
            effect: Effect::Noop,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        };
        let label = ability_cost_label(&petal);
        assert!(label.contains("{T}") && label.contains("Sac"),
            "{label} = `{{T}}, Sac`-style for Lotus Petal");
    }

    /// `sac_other_filter` / `tap_other_filter` additional costs must show
    /// in the tooltip so the ability doesn't look free for tap+mana alone.
    #[test]
    fn ability_cost_label_renders_sac_other_and_tap_other_riders() {
        use crate::card::SelectionRequirement as R;
        use crate::effect::{ActivatedAbility, Effect};
        // "{T}, Sacrifice a creature: ..." (a sac-outlet).
        let sac_outlet = ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::ControlledByYou), 1)),
            ..Default::default()
        };
        let label = ability_cost_label(&sac_outlet);
        assert!(label.contains("Sacrifice a creature"), "got: {label}");

        // "Tap an untapped creature you control: ..." (a tap-outlet).
        let tap_outlet = ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_other_filter: Some(R::Creature.and(R::Untapped)),
            effect: Effect::Noop,
            ..Default::default()
        };
        assert!(
            ability_cost_label(&tap_outlet).contains("Tap a creature"),
            "tap-other rider should render"
        );

        // Plural sac count.
        let sac_two = ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            sac_other_filter: Some((R::Artifact, 2)),
            ..Default::default()
        };
        assert!(ability_cost_label(&sac_two).contains("Sacrifice 2 artifacts"));
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
        // Accept either "instant/sorcery" or "spell" wording so the label
        // can evolve without breaking the test.
        let lab = &lifegain.gate_label;
        assert!(lab.contains("instant/sorcery") || lab.contains("spell"),
            "gate_label should describe the predicate (got {:?})", lab);
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

    /// The command zone is a public zone — every viewer sees every
    /// seat's commanders as `Known`, including opponents'. Surfaces
    /// the Phase J seating + Phase L cast-tax UI requirements.
    #[test]
    fn command_zone_is_publicly_visible_to_all_viewers() {
        let mut state = two_player_game();
        let cmd_ids = state.seat_commanders(0, vec![catalog::atraxa_grand_unifier()]);
        let atraxa = cmd_ids[0];

        // Viewer is the commander owner.
        let view_p0 = project(&state, 0);
        assert_eq!(view_p0.players[0].command.len(), 1);
        match &view_p0.players[0].command[0] {
            HandCardView::Known(k) => assert_eq!(k.id, atraxa),
            HandCardView::Hidden { .. } => panic!("own command zone must be Known"),
        }
        assert!(view_p0.players[0].commanders.contains(&atraxa));

        // Opponent viewer — still Known, because the command zone is
        // public.
        let view_p1 = project(&state, 1);
        assert_eq!(view_p1.players[0].command.len(), 1);
        match &view_p1.players[0].command[0] {
            HandCardView::Known(k) => assert_eq!(k.id, atraxa),
            HandCardView::Hidden { .. } => panic!("opponent's command zone is also public"),
        }
        // Commanders list also visible to opponents — needed so the
        // UI can flag opponents' commanders on the battlefield for
        // damage-tally tooltips.
        assert!(view_p1.players[0].commanders.contains(&atraxa));
    }

    /// Commander damage recorded in the engine surfaces in the victim's
    /// `PlayerView`, resolved to the source commander's name + owning seat
    /// (CR 903.10a). The non-victim seat shows none.
    #[test]
    fn commander_damage_taken_surfaces_in_view() {
        let mut state = two_player_game();
        let cmd_ids = state.seat_commanders(0, vec![catalog::atraxa_grand_unifier()]);
        let atraxa = cmd_ids[0];
        // Seat 0's commander has dealt 14 combat damage to seat 1.
        state.commander_damage.insert((1, atraxa), 14);

        let view = project(&state, 1);
        let victim = &view.players[1];
        assert_eq!(victim.commander_damage_taken.len(), 1);
        let entry = &victim.commander_damage_taken[0];
        assert_eq!(entry.amount, 14);
        assert_eq!(entry.source_seat, 0, "Atraxa is owned by seat 0");
        assert!(
            entry.source_name.contains("Atraxa"),
            "expected resolved commander name, got {}",
            entry.source_name
        );

        // The seat that dealt the damage has taken none itself.
        assert!(view.players[0].commander_damage_taken.is_empty());
    }

    /// Multiple source commanders are listed separately and sorted with the
    /// closest-to-lethal first (each is its own CR 903.10a clock).
    #[test]
    fn commander_damage_lists_each_source_highest_first() {
        let mut state = two_player_game();
        let a = state.seat_commanders(0, vec![catalog::atraxa_grand_unifier()])[0];
        let b = state.seat_commanders(1, vec![catalog::atraxa_grand_unifier()])[0];
        // Two different commanders have hit seat 0 for different totals.
        state.commander_damage.insert((0, a), 6);
        state.commander_damage.insert((0, b), 17);

        let view = project(&state, 0);
        let taken = &view.players[0].commander_damage_taken;
        assert_eq!(taken.len(), 2);
        assert_eq!(taken[0].amount, 17, "highest tally must lead");
        assert_eq!(taken[1].amount, 6);
    }

    #[test]
    fn trigger_event_label_covers_another_attacks() {
        // Sparring Regimen's "whenever a creature you control attacks"
        // trigger is scoped `AnotherOfYours` on `EventKind::Attacks`.
        // The view should surface this as "Another attacks: …" so the
        // client tooltip renders the printed Oracle nicely. Push this
        // run: lock the label so future label refactors can't drop it.
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::sparring_regimen());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert!(
            perm.triggered_ability_labels.iter().any(|s| s.starts_with("Another attacks")),
            "expected 'Another attacks' label for Sparring Regimen's Attacks/AnotherOfYours trigger; got {:?}",
            perm.triggered_ability_labels,
        );
    }

    #[test]
    fn permanent_view_has_mana_cost_and_creature_types() {
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert_eq!(perm.mana_cost_display, "{1}{G}");
        assert!(perm.creature_types.contains(&"Bear".to_string()));
    }

    #[test]
    fn trigger_event_label_covers_gy_leaves_your_control() {
        // Spirit Mascot-style "whenever one or more cards leave your
        // graveyard" should render as "GY leaves" in the view label.
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::spirit_mascot());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert!(
            perm.triggered_ability_labels.iter().any(|s| s.starts_with("GY leaves")),
            "expected 'GY leaves' label for Spirit Mascot's CardLeftGraveyard trigger; got {:?}",
            perm.triggered_ability_labels,
        );
    }

    #[test]
    fn exile_card_view_surfaces_mana_value_and_token_flag() {
        // Push (modern_decks): the ExileCardView now carries mana_value,
        // is_token, and may_play_recipient so the client can render an
        // exile browser tooltip without re-fetching CardDefinition.
        let mut state = two_player_game();
        // Stash a Lightning Bolt directly in exile (no may-play grant).
        let bolt_def = catalog::lightning_bolt();
        let bolt_id = state.next_id();
        let mut bolt = crate::card::CardInstance::new(bolt_id, bolt_def, 0);
        bolt.controller = 0;
        state.exile.push(bolt);

        let view = project(&state, 0);
        let entry = view.exile.iter().find(|c| c.id == bolt_id).expect("bolt in exile");
        // Lightning Bolt costs {R}, so CMC = 1.
        assert_eq!(entry.mana_value, 1);
        // Plain CardInstance, not a token.
        assert!(!entry.is_token);
        // No may-play grant — recipient is None.
        assert_eq!(entry.may_play_recipient, None);
        // Not a linked exile.
        assert_eq!(entry.exiled_by, None);
    }

    #[test]
    fn exile_card_view_surfaces_linked_exile_source() {
        // A card exiled "until ~ leaves the battlefield" carries the
        // linking source's CardId so the client can tether it.
        let mut state = two_player_game();
        let src = crate::card::CardId(4242);
        let bolt_id = state.next_id();
        let mut bolt = crate::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
        bolt.exiled_by = Some(crate::card::ExileLink {
            source: src,
            return_to: crate::card::ExileReturnZone::Hand,
        });
        state.exile.push(bolt);
        let view = project(&state, 0);
        let entry = view.exile.iter().find(|c| c.id == bolt_id).expect("bolt in exile");
        assert_eq!(entry.exiled_by, Some(src));
    }

    #[test]
    fn exile_card_view_surfaces_may_play_recipient() {
        // When an exile card carries a may_play_until permission (e.g.
        // Conspiracy Theorist's exile-top), the recipient seat surfaces
        // through the view so the client can paint a "may play" badge.
        let mut state = two_player_game();
        let bolt_def = catalog::lightning_bolt();
        let bolt_id = state.next_id();
        let mut bolt = crate::card::CardInstance::new(bolt_id, bolt_def, 0);
        bolt.controller = 0;
        bolt.may_play_until = Some(crate::card::MayPlayPermission {
            player: 0,
            granted_turn: 1,
            duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
            exile_after: false,
        });
        state.exile.push(bolt);

        let view = project(&state, 0);
        let entry = view.exile.iter().find(|c| c.id == bolt_id).expect("bolt in exile");
        assert_eq!(entry.may_play_recipient, Some(0));
    }

    #[test]
    fn project_surfaces_emblem_names() {
        let mut state = two_player_game();
        state.players[0].emblems.push(crate::player::Emblem {
            name: "Professor Dellian Fel".into(),
            triggered: vec![],
        });
        let view = project(&state, 0);
        assert_eq!(view.players[0].emblems, vec!["Professor Dellian Fel".to_string()]);
    }

    #[test]
    fn known_card_distinguishes_pitch_from_plain_alt_cost() {
        // Pyrokinesis exiles a red card (pitch) → needs_pitch = true.
        let pitch = crate::card::CardInstance::new(
            crate::card::CardId(1), catalog::pyrokinesis(), 0);
        let k = known_card(&pitch);
        assert!(k.has_alternative_cost);
        assert!(k.alt_cost_needs_pitch, "Pyrokinesis pitches a card");

        // Boulder Salvo's Surge is a plain alt cost (no exile) with a label.
        let surge = crate::card::CardInstance::new(
            crate::card::CardId(2), catalog::boulder_salvo(), 0);
        let k2 = known_card(&surge);
        assert!(k2.has_alternative_cost);
        assert!(!k2.alt_cost_needs_pitch, "Surge needs no pitch");
        assert_eq!(k2.alt_cost_label, "{1}{R}", "surge cost label rendered");
    }
}
