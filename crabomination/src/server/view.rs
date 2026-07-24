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
    project_for(state, Some(seat))
}

/// Project the authoritative `state` for a read-only spectator: a viewer who
/// occupies no seat. Every player's hand and library is hidden, no
/// pending-decision contents are revealed, and no cast/attack/block
/// affordances are computed (a spectator can't act). `your_seat` is set to
/// [`crate::net::SPECTATOR_SEAT`] so the client renders read-only.
pub fn project_spectator(state: &GameState) -> ClientView {
    project_for(state, None)
}

/// Shared projection core. `viewer` is `Some(seat)` for a seated player or
/// `None` for a spectator (no seat, sees only public information).
fn project_for(state: &GameState, viewer: Option<usize>) -> ClientView {
    // The projection reads the layer system many times (battlefield rows,
    // legal attackers/blockers, combat preview) — share one gather. Dry-run
    // affordance probes mutate *clones*, which start unfrozen.
    state.with_frozen_layers(|state| project_for_inner(state, viewer))
}

fn project_for_inner(state: &GameState, viewer: Option<usize>) -> ClientView {
    let computed = state.compute_battlefield();
    // A spectator can't act, so skip the affordance dry-runs entirely (they
    // index `state.players[seat]` and would panic on the sentinel anyway).
    let affordances = match viewer {
        Some(seat) => state.compute_hand_affordances(seat),
        None => crate::game::HandAffordances::default(),
    };
    // Sentinel viewer seat for hand-visibility checks: no owner seat equals
    // it, so `project_hand_card` hides every hand for a spectator.
    let viewer_seat = viewer.unwrap_or(crate::net::SPECTATOR_SEAT);

    ClientView {
        your_seat: viewer_seat,
        active_player: state.active_player_idx,
        priority: state.player_with_priority(),
        step: state.step,
        turn: state.turn_number,
        extra_phase: is_extra_phase(state),
        players: state
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| {
                use crate::mana::Color;
                let devotion = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
                    .map(|c| state.devotion_to(i, &[c]).max(0) as u32);
                // Hexproof is surfaced from the viewer's perspective: if the
                // viewer controls a broad "ignore opponents' hexproof" static
                // (Kaya, Bane of the Dead), an opponent's hexproof no longer
                // shields them from the viewer, so the flag reads false and the
                // client won't grey the player out as a target.
                let hexproof = state.player_has_static_hexproof(i)
                    && !(i != viewer_seat && state.player_ignores_hexproof(viewer_seat));
                project_player(state, p, i, viewer_seat, &state.prevention_shields, devotion, state.draw_cap_for(i), state.monarch == Some(i), commander_damage_taken(state, i), state.team_of(i).0, state.player_cannot_gain_life_now(i), hexproof, known_library_top(state, i, viewer_seat))
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
                        viewer_seat,
                        state,
                    )
                })
                .collect()
        },
        stack: state
            .stack
            .iter()
            .map(|item| project_stack(item, state, viewer_seat))
            .collect(),
        pending_decision: state.pending_decision.as_ref().map(|pd| {
            let acting = pd.acting_player();
            PendingDecisionView {
                acting_player: acting,
                // Only the acting player sees decision specifics; spectators
                // see that someone is deciding but not the private contents.
                decision: (viewer == Some(acting)).then(|| (&pd.decision).into()),
            }
        }),
        exile: state.exile.iter().map(|c| exile_entry(c, viewer)).collect(),
        game_over: state.game_over,
        damage_cant_be_prevented_this_turn: state.damage_cant_be_prevented_this_turn,
        combat_damage_prevented_this_turn: state.prevent_combat_damage_this_turn,
        day_night: state.day_night.map(|dn| dn == crate::game::types::DayNight::Day),
        combat_preview: combat_preview(state),
        // One pass over the hand builds a single library-stripped probe
        // template and reuses it across every affordance category, rather
        // than each `*_hand_cards` call cloning the whole `GameState` (incl.
        // both libraries) per candidate. Runs on every accepted action for
        // the priority-holding seat, so it's the projection's hot path.
        castable_hand: affordances.castable,
        back_castable_hand: affordances.back_castable,
        prepare_castable: affordances.prepare_castable,
        pitchable_hand: affordances.pitchable,
        kickable_hand: affordances.kickable,
        buyback_hand: affordances.buyback,
        bestowable_hand: affordances.bestowable,
        miracle_hand: affordances.miracle,
        bargainable_hand: affordances.bargainable,
        squadable_hand: affordances.squadable,
        spreeable_hand: affordances.spreeable,
        replicatable_hand: affordances.replicatable,
        conspirable_hand: affordances.conspirable,
        multikickable_hand: affordances.multikickable,
        dashable_hand: affordances.dashable,
        blitzable_hand: affordances.blitzable,
        warpable_hand: affordances.warpable,
        suspendable_hand: affordances.suspendable,
        foretellable_hand: affordances.foretellable,
        plottable_hand: affordances.plottable,
        adventurable_hand: affordances.adventurable,
        omenable_hand: affordances.omenable,
        prototypable_hand: affordances.prototypable,
        splittable_right_hand: affordances.splittable_right,
        activatable_permanents: affordances.activatable_permanents,
        hand_activatable: affordances.hand_activatable,
        morphable_hand: affordances.morphable,
        turn_up_able: affordances.turn_up_able,
        reinforceable_hand: affordances.reinforceable,
        discard_activatable_hand: affordances.discard_activatable,
        room_castable_hand: affordances.room_castable,
        room_unlockable: affordances.room_unlockable,
        legal_attackers: viewer.map(|s| state.legal_attackers(s)).unwrap_or_default(),
        legal_blockers: viewer.map(|s| state.legal_blockers(s)).unwrap_or_default(),
        permanents_to_graveyard_this_turn: state.permanents_to_graveyard_this_turn,
    }
}

/// CR 500.7 — true when the current step is a *repeated* phase this turn: the
/// active player is in an additional combat phase (any combat step while more
/// than one combat has begun) or an additional end step. Drives the phase-bar
/// "extra" marker so a looped combat/end step reads clearly.
pub(crate) fn is_extra_phase(state: &GameState) -> bool {
    use crate::game::types::TurnStep::*;
    let in_combat = matches!(
        state.step,
        BeginCombat | DeclareAttackers | DeclareBlockers | FirstStrikeDamage | CombatDamage | EndCombat
    );
    (in_combat && state.combat_phases_this_turn > 1)
        || (state.step == crate::game::types::TurnStep::End && state.end_steps_this_turn > 1)
}

/// Total Afflict N across a card's triggered abilities (CR 702.131) — a
/// self-source "becomes blocked" trigger that drains the defending player.
/// Returns 0 for cards without Afflict. Used by the combat preview so the
/// HUD reflects the on-block life loss.
fn afflict_amount(def: &crate::card::CardDefinition) -> i32 {
    use crate::card::{EventKind, EventScope};
    use crate::effect::{Effect, PlayerRef, Selector, Value};
    def.triggered_abilities
        .iter()
        .filter(|ta| {
            ta.event.kind == EventKind::BecomesBlocked
                && matches!(ta.event.scope, EventScope::SelfSource)
        })
        .filter_map(|ta| match &ta.effect {
            Effect::LoseLife { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::Const(n) } => Some(*n),
            _ => None,
        })
        .sum()
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
    let mut pw_dmg: std::collections::HashMap<CardId, i32> = std::collections::HashMap::new();
    let mut lifegain: std::collections::HashMap<usize, i32> = std::collections::HashMap::new();
    let mut dying: Vec<CardId> = Vec::new();

    // Use layer-computed P/T + keywords so the preview honors anthems,
    // granted/stripped evasion (e.g. a granted Trample or Deathtouch), and
    // keyword loss — not just the printed/counter values on the instance.
    type CP = crate::game::layers::ComputedPermanent;
    let computed = state.compute_battlefield();
    let cp = |id: CardId| computed.iter().find(|c| c.id == id);
    let kw = |c: &CP, k: &Keyword| c.keywords.contains(k);

    let lethal_from = |attacker: &CP, defender: &CP| -> bool {
        let p = attacker.power;
        p > 0
            && !kw(defender, &Keyword::Indestructible)
            // CR 702.16e — combat damage from a color the defender has
            // protection from is prevented, so it never dies to that source.
            && !state.damage_prevented_by_protection(attacker.id, defender.id)
            && (p >= defender.toughness || kw(attacker, &Keyword::Deathtouch))
    };

    for atk in attackers {
        let Some(a) = cp(atk.attacker) else { continue };
        let blockers: Vec<&CP> = block_map
            .iter()
            .filter(|(_, aid)| *aid == atk.attacker)
            .filter_map(|(bid, _)| cp(*bid))
            .collect();
        let a_power = a.power.max(0);
        let lifelink = kw(a, &Keyword::Lifelink);
        // CR 702.4 — a double striker deals its combat damage twice (the
        // first-strike step *and* the regular step), so unblocked face damage,
        // trample overflow, and lifelink all count it twice.
        let strikes = if kw(a, &Keyword::DoubleStrike) { 2 } else { 1 };
        if blockers.is_empty() {
            // CR 510.1c — a blocked attacker whose blockers all left combat
            // stays blocked: no face damage without trample.
            if state.blocked_attackers().contains(&atk.attacker) && !kw(a, &Keyword::Trample) {
                continue;
            }
            // Unblocked: full damage to the defending player or planeswalker
            // (×2 for double strike).
            let face = a_power * strikes;
            match atk.target {
                AttackTarget::Player(p) => {
                    *dmg.entry(p).or_insert(0) += face;
                }
                AttackTarget::Planeswalker(pw) => {
                    *pw_dmg.entry(pw).or_insert(0) += face;
                }
                // Battle damage removes defense counters, not life — the board
                // view shows the live counter total, so nothing to predict here.
                AttackTarget::Battle(_) => {}
            }
            if lifelink && face > 0 {
                *lifegain.entry(a.controller).or_insert(0) += face;
            }
        } else {
            // Blocked: attacker assigns lethal to blockers in id order;
            // trample overflows to the defending player.
            let has_fs = |c: &CP| kw(c, &Keyword::FirstStrike) || kw(c, &Keyword::DoubleStrike);
            let attacker_fs = has_fs(a);
            // Which blockers does the attacker kill? (lethal spread, deathtouch
            // first-blocker-eats-all). Computed first so first-strike removal
            // can suppress those blockers' damage back.
            let mut remaining = a_power;
            let mut killed: Vec<CardId> = Vec::new();
            for b in &blockers {
                let needed = b.toughness.max(1);
                if lethal_from(a, b) && (kw(a, &Keyword::Deathtouch) || remaining >= needed) {
                    killed.push(b.id);
                    remaining -= if kw(a, &Keyword::Deathtouch) { 1 } else { needed };
                }
            }
            // CR 702.7 — a non-first-strike blocker the attacker kills in the
            // first-strike step deals no damage back. Such blockers don't
            // count toward the attacker's death.
            let deals_back =
                |b: &CP| !(attacker_fs && !has_fs(b) && killed.contains(&b.id));
            let total_blocker_power: i32 =
                blockers.iter().filter(|b| deals_back(b)).map(|b| b.power.max(0)).sum();
            let dt_blocker = blockers
                .iter()
                .any(|b| b.power > 0 && kw(b, &Keyword::Deathtouch) && deals_back(b));
            if (total_blocker_power > 0 || dt_blocker)
                && !kw(a, &Keyword::Indestructible)
                && (total_blocker_power >= a.toughness || dt_blocker)
            {
                dying.push(a.id);
            }
            for bid in &killed {
                dying.push(*bid);
            }
            // Trample overflow (CR 510.1c): leftover after lethal to all
            // blockers spills to the defending player.
            if kw(a, &Keyword::Trample) {
                let assign_to_block: i32 = blockers
                    .iter()
                    .map(|b| if kw(a, &Keyword::Deathtouch) { 1 } else { b.toughness.max(0) })
                    .sum();
                let overflow = (a_power - assign_to_block).max(0);
                // Double strike (CR 702.4): a second damage step. If the first
                // strike killed every blocker, the whole power tramples through;
                // otherwise the survivors soak the same lethal again.
                let second_overflow = if strikes == 2 {
                    if killed.len() == blockers.len() { a_power } else { overflow }
                } else {
                    0
                };
                let total_overflow = overflow + second_overflow;
                if total_overflow > 0 {
                    match atk.target {
                        AttackTarget::Player(p) => {
                            *dmg.entry(p).or_insert(0) += total_overflow;
                        }
                        AttackTarget::Planeswalker(pw) => {
                            *pw_dmg.entry(pw).or_insert(0) += total_overflow;
                        }
                        AttackTarget::Battle(_) => {}
                    }
                }
            }
            // CR 702.131 — Afflict: a blocked attacker drains the defending
            // player (life loss, surfaced as predicted player-life change).
            if let AttackTarget::Player(p) = atk.target {
                let afflict = state
                    .battlefield_find(atk.attacker)
                    .map(|c| afflict_amount(&c.definition))
                    .unwrap_or(0);
                if afflict > 0 {
                    *dmg.entry(p).or_insert(0) += afflict;
                }
            }
            if lifelink {
                // A double striker deals (and so lifelinks for) its power twice.
                *lifegain.entry(a.controller).or_insert(0) += a_power * strikes;
            }
            // Blockers with lifelink gain their controller life for the
            // damage they deal to the attacker (a first-struck-dead blocker
            // deals none).
            for b in &blockers {
                if kw(b, &Keyword::Lifelink) && deals_back(b) {
                    *lifegain.entry(b.controller).or_insert(0) += b.power.max(0);
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
    let mut damage_to_planeswalkers: Vec<(CardId, i32)> =
        pw_dmg.into_iter().filter(|(_, d)| *d != 0).collect();
    damage_to_planeswalkers.sort();
    Some(crate::net::CombatPreview {
        damage_to_players,
        lifegain_to_players,
        dying_creatures: dying,
        damage_to_planeswalkers,
    })
}

fn exile_entry(card: &CardInstance, viewer: Option<usize>) -> ExileCardView {
    // CR 708 — a face-down exiled card (hideaway, foretell) is hidden from
    // everyone but its controller: mask the identity.
    let hidden = card.face_down && viewer != Some(card.controller);
    ExileCardView {
        id: card.id,
        name: if hidden { "Face-down card".to_string() } else { card.definition.name.to_string() },
        owner: card.owner,
        may_play_recipient: card.may_play_until.as_ref().map(|p| p.player),
        // Surface the alt-cast cost only while a may-play grant is live, so
        // "play for {2}" can't leak from a stale cost on a plain exile card.
        may_play_alt_cost: card
            .may_play_until
            .as_ref()
            .and(card.granted_alt_cast_cost_eot.as_ref())
            .map(|c| c.cmc()),
        mana_value: if hidden { 0 } else { card.definition.cost.cmc() },
        is_token: card.is_token,
        exiled_by: card.exiled_by.map(|l| l.source),
        encoded_on: card.encoded_on,
        face_down: card.face_down,
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

/// CR 401.5/401.6 — the top library card is public while a
/// `TopOfLibraryRevealed` static is active (Courser of Kruphix), and
/// owner-visible under a `PlayFromLibraryTop` permission (Mystic Forge's
/// "may look at the top card of your library any time").
fn known_library_top(
    state: &GameState,
    player_seat: usize,
    viewer_seat: usize,
) -> Vec<crate::net::KnownCard> {
    use crate::effect::StaticEffect;
    let has_static = |pred: &dyn Fn(&StaticEffect) -> bool| {
        state.battlefield.iter().any(|c| {
            c.controller == player_seat
                && c.definition.static_abilities.iter().any(|sa| pred(&sa.effect))
        })
    };
    // Lantern of Insight — any permanent with the all-players static
    // reveals every library top, regardless of controller.
    let lantern = state.battlefield.iter().any(|c| {
        c.definition
            .static_abilities
            .iter()
            .any(|sa| matches!(sa.effect, StaticEffect::AllLibraryTopsRevealed))
    });
    let revealed_to_all =
        lantern || has_static(&|e| matches!(e, StaticEffect::TopOfLibraryRevealed));
    let owner_may_look = viewer_seat == player_seat
        && (has_static(&|e| matches!(e, StaticEffect::PlayFromLibraryTop { .. }
                | StaticEffect::PlayFromLibraryTopOncePerTurn { .. }
                | StaticEffect::PlayFromLibraryTopPayLife { .. }))
            || state.players[player_seat].play_from_top_this_turn);
    if revealed_to_all || owner_may_look {
        state.players[player_seat].library.first().map(known_card).into_iter().collect()
    } else {
        Vec::new()
    }
}

#[allow(clippy::too_many_arguments)]
fn project_player(
    state: &GameState,
    player: &Player,
    player_seat: usize,
    viewer_seat: usize,
    prevention_shields: &[crate::game::types::PreventionShield],
    devotion: [u32; 5],
    draw_cap: Option<u32>,
    is_monarch: bool,
    commander_damage_taken: Vec<crate::net::CommanderDamageEntry>,
    team: usize,
    cannot_gain_life: bool,
    has_hexproof: bool,
    known_top: Vec<crate::net::KnownCard>,
) -> PlayerView {
    use crate::game::types::PreventionTarget;
    let has_prevention_shield = prevention_shields
        .iter()
        .any(|s| s.target == PreventionTarget::Player(player_seat));
    let damage_fully_prevented = state.all_damage_to_player_prevented(player_seat);
    // Coven — three or more controlled creatures with different (computed) powers.
    let coven_active = {
        let powers: std::collections::HashSet<i32> = state
            .battlefield
            .iter()
            .filter(|c| c.controller == player_seat && c.definition.is_creature())
            .filter_map(|c| state.computed_permanent(c.id).map(|cp| cp.power))
            .collect();
        powers.len() >= 3
    };
    // Ability-word conditions (mirror `crate::game::effects::eval`).
    let threshold_active = player.graveyard.len() >= 7;
    let metalcraft_active = state
        .battlefield
        .iter()
        .filter(|c| c.controller == player_seat && c.definition.is_artifact())
        .count()
        >= 3;
    let controlled_creature_powers = || {
        state
            .battlefield
            .iter()
            .filter(|c| c.controller == player_seat && c.definition.is_creature())
            .filter_map(|c| state.computed_permanent(c.id).map(|cp| cp.power))
    };
    let ferocious_active = controlled_creature_powers().any(|p| p >= 4);
    let formidable_active = controlled_creature_powers().sum::<i32>() >= 8;
    let hellbent_active = player.hand.is_empty();
    // CR 611.2 — per-turn spell-cast locks in play, and whether this player has
    // already cast a spell of each locked category this turn.
    let spell_cast_lock = {
        use crate::effect::StaticEffect;
        let any_static = |pred: &dyn Fn(&StaticEffect) -> bool| {
            state
                .battlefield
                .iter()
                .any(|c| c.definition.static_abilities.iter().any(|sa| pred(&sa.effect)))
        };
        crate::net::SpellCastLock {
            any_reached: player.spells_cast_this_game_turn >= 1
                && any_static(&|e| matches!(e, StaticEffect::OneSpellPerTurn)),
            noncreature_reached: player.noncreature_spells_cast_this_game_turn >= 1
                && any_static(&|e| matches!(e, StaticEffect::OneNoncreatureSpellPerTurn)),
            nonartifact_reached: player.nonartifact_spells_cast_this_game_turn >= 1
                && any_static(&|e| matches!(e, StaticEffect::OneNonartifactSpellPerTurn)),
            creature_pw_locked: !state.creature_pw_cast_locks.is_empty(),
        }
    };
    // CR 601.3e — an opponent's Void Winnower locks this player's even-MV casts.
    let even_mv_cast_locked = state.battlefield.iter().any(|c| {
        !state.same_team(c.controller, player_seat)
            && c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::OpponentsCantCastEvenMv))
    });
    PlayerView {
        seat: player_seat,
        name: player.name.clone(),
        life: player.life,
        starting_life: player.starting_life,
        poison_counters: player.poison_counters,
        energy: player.energy,
        experience: player.experience,
        speed: player.speed,
        at_max_speed: player.speed >= 4,
        rad_counters: player.rad_counters,
        mana_pool: player.mana_pool.clone(),
        kept_mana: player.kept_mana_this_turn.total(),
        library: LibraryView {
            size: player.library.len(),
            known_top,
        },
        graveyard: player
            .graveyard
            .iter()
            .map(|c| graveyard_entry(c, state, player_seat))
            .collect(),
        phased_out: state
            .phased_out
            .iter()
            .filter(|c| c.controller == player_seat)
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect(),
        hand: player
            .hand
            .iter()
            .map(|c| project_hand_card(c, state, player_seat, viewer_seat))
            .collect(),
        lands_played_this_turn: player.lands_played_this_turn,
        first_spell_tax_charges: player.first_spell_tax_charges,
        life_gained_this_turn: player.life_gained_this_turn,
        cards_drawn_this_turn: player.cards_drawn_this_turn,
        draw_cap,
        cards_left_graveyard_this_turn: player.cards_left_graveyard_this_turn,
        creatures_died_this_turn: player.creatures_died_this_turn,
        next_creature_bonus_counters: player
            .pending_creature_etb_counters
            .iter()
            .filter(|(k, _)| *k == crate::card::CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n)
            .sum(),
        next_creature_gains_haste: player
            .pending_creature_etb_keywords
            .contains(&crate::card::Keyword::Haste),
        cards_exiled_this_turn: player.cards_exiled_this_turn,
        instants_or_sorceries_cast_this_turn: player.instants_or_sorceries_cast_this_turn,
        creatures_cast_this_turn: player.creatures_cast_this_turn,
        spells_cast_this_turn: player.spells_cast_this_turn,
        spell_cast_lock,
        even_mv_cast_locked,
        skip_next_combat: player.skip_next_combat,
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
        // Tax tally per commander (CR 903.8). Resolve each id to a name the
        // same way `commander_damage_taken` does: any tracked zone first,
        // then the command zones (which `find_card_anywhere` skips).
        commander_casts: player
            .commanders
            .iter()
            .map(|id| {
                let name = state
                    .find_card_anywhere(*id)
                    .or_else(|| {
                        state
                            .players
                            .iter()
                            .find_map(|p| p.command.iter().find(|c| c.id == *id))
                    })
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_else(|| "Commander".to_string());
                (name, state.commander_cast_count.get(id).copied().unwrap_or(0))
            })
            .collect(),
        eliminated: player.eliminated,
        loss_reason: player.loss_cause.map(|c| {
            match c {
                crate::player::LossCause::LifeDepleted => "life",
                crate::player::LossCause::Poison => "poison",
                crate::player::LossCause::Decked => "decked",
                crate::player::LossCause::CommanderDamage => "commander",
                crate::player::LossCause::Conceded => "conceded",
                crate::player::LossCause::Other => "lose effect",
            }
            .to_string()
        }),
        // Emblem label = source name plus any static-ability text, so the UI
        // can show what an anthem emblem (Vivien Reid's −8) actually does
        // rather than just its name. Triggered-only emblems keep the bare name.
        emblems: player
            .emblems
            .iter()
            .map(|e| {
                if e.statics.is_empty() {
                    e.name.clone()
                } else {
                    let text = e
                        .statics
                        .iter()
                        .map(|s| s.description)
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("{} — {}", e.name, text)
                }
            })
            .collect(),
        has_prevention_shield,
        damage_fully_prevented,
        devotion,
        is_monarch,
        has_city_blessing: player.city_blessing,
        cannot_gain_life,
        cant_cast_noncreature: player.cant_cast_noncreature_this_turn,
        life_locked: player.life_locked_this_turn,
        has_hexproof,
        commander_damage_taken,
        team,
        dungeon: state.players[player_seat].dungeon.as_ref().and_then(|(name, room)| {
            let def = crabomination_base::dungeons::dungeon_by_name(name)?;
            Some((name.clone(), def.rooms.get(*room as usize)?.name.to_string()))
        }),
        dungeons_completed: state.players[player_seat].dungeons_completed,
        coven_active,
        descend_count: state.players[player_seat]
            .graveyard
            .iter()
            .filter(|c| c.definition.is_permanent())
            .count() as u32,
        descended_this_turn_count: state.players[player_seat].descend_count_this_turn,
        committed_crime_this_turn: state.players[player_seat].committed_crime_this_turn,
        ring_temptations: player.ring_temptations,
        ring_bearer: state.effective_ring_bearer(player_seat),
        void_active: state.nonland_permanent_left_bf_this_turn
            || state.players[player_seat].warped_spell_this_turn,
        threshold_active,
        metalcraft_active,
        ferocious_active,
        hellbent_active,
        formidable_active,
        mazes_end_gate_progress: state
            .battlefield
            .iter()
            .any(|c| c.controller == player_seat && c.definition.name == "Maze's End")
            .then(|| {
                let mut names: Vec<&str> = state
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == player_seat
                            && c.definition.has_land_type(crate::card::LandType::Gate)
                    })
                    .map(|c| c.definition.name)
                    .collect();
                names.sort_unstable();
                names.dedup();
                names.len() as u32
            }),
    }
}

fn project_hand_card(
    card: &CardInstance,
    state: &crate::game::GameState,
    owner_seat: usize,
    viewer_seat: usize,
) -> HandCardView {
    if owner_seat == viewer_seat {
        HandCardView::Known(known_card_in(card, Some(state)))
    } else {
        HandCardView::Hidden { id: card.id }
    }
}

fn known_card(card: &CardInstance) -> KnownCard {
    known_card_in(card, None)
}

fn known_card_in(card: &CardInstance, state: Option<&crate::game::GameState>) -> KnownCard {
    let cycling_cost = card.definition.keywords.iter().find_map(|kw| {
        if let crate::card::Keyword::Cycling(c) = kw {
            Some(c.clone())
        } else {
            None
        }
    });
    let cycling_life = card.definition.keywords.iter().find_map(|kw| {
        if let crate::card::Keyword::CyclingLife(n) = kw { Some(*n) } else { None }
    });
    let landcycling_cost = card
        .definition
        .keywords
        .iter()
        .find_map(|kw| {
            // Typecycling rides the same client affordance (CR 702.29e).
            match kw {
                crate::card::Keyword::Landcycling(c, _) => Some(c.clone()),
                crate::card::Keyword::Typecycling(spec) => Some(spec.0.clone()),
                _ => None,
            }
        })
        // Battlefield-granted typecycling (Homing Sliver's slivercycling).
        .or_else(|| state.and_then(|st| st.granted_typecycling_for(card)).map(|(c, _)| c));
    let (modal_descriptions, modal_needs_target, modal_target_optional) =
        if let crate::effect::Effect::ChooseMode(modes) = &card.definition.effect {
            let descs = modes.iter().map(|m| m.effect_short_text()).collect();
            let needs = modes.iter().map(cursor_needs_target).collect();
            let opt = modes.iter().map(|m| m.target_slot_optional(0, None)).collect();
            (descs, needs, opt)
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };
    KnownCard {
        id: card.id,
        name: card.definition.name.to_string(),
        cost: card.definition.cost.clone(),
        card_types: card.definition.card_types.clone(),
        needs_target: cursor_needs_target(&card.definition.effect),
        target_optional: card.definition.effect.target_slot_optional(0, None),
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
            .map(format_alt_cost_label)
            .unwrap_or_default(),
        alt_cost_available: card.definition.alternative_cost.as_ref().is_none_or(|a| {
            // Condition-gated alt costs (Prowl, Archive Trap) and
            // not-your-turn pitches grey out when unavailable; without a
            // GameState handle (command-zone views) report available.
            let Some(st) = state else { return true };
            let cond_ok = a.condition.as_ref().is_none_or(|c| {
                let ctx = crate::game::effects::EffectContext::for_ability(
                    crate::card::CardId(0),
                    card.owner,
                    None,
                );
                st.evaluate_predicate(c, &ctx)
            });
            // CR 702.48 — Offering greys out unless the caster controls a
            // creature of the offered type to sacrifice.
            let offering_ok = a.offering.as_ref().is_none_or(|filter| {
                st.battlefield.iter().any(|c| {
                    c.controller == card.owner
                        && c.definition.is_creature()
                        && st.evaluate_requirement_static(
                            filter,
                            &crate::game::types::Target::Permanent(c.id),
                            card.owner,
                            None,
                        )
                })
            });
            // A return-to-hand / sacrifice additional cost greys out unless the
            // caster controls enough matching permanents to pay it (Sneak needs
            // an unblocked attacker, Web-slinging a tapped creature, Fireblast
            // two Mountains).
            let controls_at_least = |filter: &crate::card::SelectionRequirement, n: u32| {
                st.battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == card.owner
                            && st.evaluate_requirement_static(
                                filter,
                                &crate::game::types::Target::Permanent(c.id),
                                card.owner,
                                None,
                            )
                    })
                    .count() as u32
                    >= n
            };
            let return_ok = a
                .return_to_hand
                .as_ref()
                .is_none_or(|(f, n)| controls_at_least(f, *n));
            let sac_ok = a
                .sacrifice_permanents
                .as_ref()
                .is_none_or(|(f, n)| controls_at_least(f, *n));
            cond_ok
                && offering_ok
                && return_ok
                && sac_ok
                && !(a.not_your_turn_only && st.active_player_idx == card.owner)
        }),
        back_face_name: card
            .definition
            .back_face
            .as_ref()
            .map(|b| b.name.to_string()),
        back_needs_target: card
            .definition
            .back_face
            .as_ref()
            .is_some_and(|b| b.effect.requires_target()),
        has_cycling: cycling_cost.is_some() || cycling_life.is_some(),
        cycling_cost_label: cycling_cost
            .as_ref()
            .map(format_mana_cost_for_label)
            .or_else(|| cycling_life.map(|n| format!("Pay {n} life")))
            .unwrap_or_default(),
        has_landcycling: landcycling_cost.is_some(),
        landcycling_cost_label: landcycling_cost
            .as_ref()
            .map(format_mana_cost_for_label)
            .unwrap_or_default(),
        modal_descriptions,
        modal_needs_target,
        modal_target_optional,
        saga_final_chapter: card
            .definition
            .saga_chapters
            .iter()
            .map(|(n, _)| *n)
            .max(),
        split_right_cost_label: card
            .definition
            .split
            .as_ref()
            .map(|sp| format_mana_cost_for_label(&sp.right.cost))
            .unwrap_or_default(),
        split_right_needs_target: card
            .definition
            .split
            .as_ref()
            .is_some_and(|sp| sp.right.effect.requires_target()),
        split_fusable: card.definition.split.as_ref().is_some_and(|sp| sp.fuse),
        split_fused_needs_target: card.definition.split.as_ref().is_some_and(|sp| {
            sp.fuse
                && (card.definition.effect.requires_target()
                    || sp.right.effect.requires_target())
        }),
        has_gift: card.definition.gift.is_some(),
        gift_label: card
            .definition
            .gift
            .as_ref()
            .map(|g| g.label.to_string())
            .unwrap_or_default(),
        gift_needs_target: card
            .definition
            .gift
            .as_ref()
            .is_some_and(|g| g.gifted_effect.requires_target()),
        has_waterbend: card.definition.waterbend.is_some(),
        waterbend_amount: card.definition.waterbend.as_ref().and_then(|wb| match wb.amount {
            crate::effect::Value::Const(n) => Some(n.max(0) as u32),
            _ => None, // waterbend {X}: amount is the chosen X
        }),
        has_omen: card.definition.omen.is_some(),
        omen_label: card
            .definition
            .omen
            .as_ref()
            .map(|o| o.name.to_string())
            .unwrap_or_default(),
        omen_needs_target: card
            .definition
            .omen
            .as_ref()
            .is_some_and(|o| o.effect.requires_target()),
        spree_mode_labels: match &card.definition.effect {
            crate::effect::Effect::Spree { modes } | crate::effect::Effect::Tiered { modes } => {
                modes
                    .iter()
                    .map(|m| format!("{} — {}", m.cost.summary(), m.effect.effect_short_text()))
                    .collect()
            }
            _ => Vec::new(),
        },
        spree_single_mode: matches!(&card.definition.effect, crate::effect::Effect::Tiered { .. }),
        station_next_threshold: {
            let charges = card.counter_count(crate::card::CounterType::Charge);
            card.definition
                .station
                .iter()
                .map(|b| b.min)
                .filter(|&m| m > charges)
                .min()
        },
        station_charges: (!card.definition.station.is_empty())
            .then(|| card.counter_count(crate::card::CounterType::Charge)),
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

/// A player-facing label for an alternative cost: the mana portion plus any
/// non-mana riders (return-to-hand / sacrifice / pitch / life), so a {0}-mana
/// alt cost like Escape Detection's "return a blue creature" still reads
/// sensibly in the cast prompt instead of showing nothing.
fn format_alt_cost_label(a: &crate::card::AlternativeCost) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mana = format_mana_cost_for_label(&a.mana_cost);
    if !mana.is_empty() {
        parts.push(mana);
    }
    if let Some((_, n)) = &a.return_to_hand {
        parts.push(format!("Return {n}"));
    }
    if let Some((_, n)) = &a.sacrifice_permanents {
        parts.push(format!("Sacrifice {n}"));
    }
    if a.exile_from_graveyard_count > 0 {
        parts.push(format!("Exile {} from graveyard", a.exile_from_graveyard_count));
    }
    if a.exile_filter.is_some() {
        parts.push("Exile a card from hand".to_string());
    }
    if a.life_cost > 0 {
        parts.push(format!("Pay {} life", a.life_cost));
    }
    parts.join(" + ")
}

fn graveyard_entry(
    card: &CardInstance,
    state: &GameState,
    seat: usize,
) -> GraveyardCardView {
    GraveyardCardView {
        id: card.id,
        name: card.definition.name.to_string(),
        card_types: card.definition.card_types.clone(),
        mana_cost: card.definition.cost.clone(),
        power: card.definition.base_power(),
        toughness: card.definition.base_toughness(),
        // Jump-start (CR 702.103) rides the flashback cast path at the
        // card's own cost (+ discard, paid at cast time). Lier grants
        // flashback (= mana cost) to I/S in the graveyard.
        flashback_cost: card
            .definition
            .has_flashback()
            .cloned()
            .or_else(|| {
                card.definition
                    .keywords
                    .contains(&crate::card::Keyword::JumpStart)
                    .then(|| card.definition.cost.clone())
            })
            .or_else(|| state.graveyard_flashback_grant(seat, card)),
        retrace: state.effective_retrace(card, seat),
        escape: state.effective_escape(card, seat),
        bestow_cost: card.definition.has_bestow().cloned(),
        buyback_cost: card.definition.has_buyback().cloned(),
        disturb_cost: card.definition.keywords.iter().find_map(|k| match k {
            crate::card::Keyword::Disturb(c) => Some(c.clone()),
            _ => None,
        }),
        // CR 702.187 — Mayhem is castable only if this seat discarded the card
        // this turn (the same gate `cast_flashback` enforces).
        mayhem_cost: card.definition.mayhem_cost().cloned().filter(|_| {
            state.players[seat].discarded_this_turn.contains(&card.id)
        }),
        harmonize_cost: card.definition.harmonize_cost().cloned(),
        scavenge_cost: state.effective_scavenge_cost(seat, card),
    }
}

/// CR 702.122e/702.171 — sum of "crews/saddles as though its power were N
/// greater" bonuses applying to `cid`, computed from a battlefield slice (the
/// view layer has no `GameState`). Mirrors `GameState::crew_saddle_power_bonus`.
fn crew_saddle_power_bonus_in(cid: CardId, battlefield: &[CardInstance]) -> i32 {
    use crate::effect::StaticEffect;
    let Some(target) = battlefield.iter().find(|c| c.id == cid) else { return 0 };
    let mut bonus = 0;
    for src in battlefield {
        for sa in &src.definition.static_abilities {
            if let StaticEffect::CrewSaddlePowerBonus { applies_to, amount } = &sa.effect
                && let Some(affected) =
                    crate::game::selector_to_affected(applies_to, src)
                && crate::game::layers::affected_includes(&affected, src.id, target)
            {
                bonus += amount;
            }
        }
    }
    bonus
}

fn project_permanent(
    card: &CardInstance,
    computed: &[crate::game::layers::ComputedPermanent],
    attacking: &[CardId],
    block_map: &[(CardId, CardId)],
    viewer_seat: usize,
    state: &crate::game::GameState,
) -> PermanentView {
    use crate::game::types::PreventionTarget;
    let battlefield = &state.battlefield;
    let cp = computed.iter().find(|c| c.id == card.id);
    let has_prevention_shield = state
        .prevention_shields
        .iter()
        .any(|s| s.target == PreventionTarget::Permanent(card.id) && !s.destroy);
    let doomed_next_damage = state
        .prevention_shields
        .iter()
        .any(|s| s.target == PreventionTarget::Permanent(card.id) && s.destroy);
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
        dealt_damage_this_turn: card.dealt_damage_this_turn,
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
        station_next_threshold: {
            let charges = card.counter_count(crate::card::CounterType::Charge);
            card.definition
                .station
                .iter()
                .map(|b| b.min)
                .filter(|&m| m > charges)
                .min()
        },
        station_charges: (!card.definition.station.is_empty())
            .then(|| card.counter_count(crate::card::CounterType::Charge)),
        attacking: attacking.contains(&card.id),
        blocking_attacker: block_map
            .iter()
            .find_map(|(b, a)| (*b == card.id).then_some(*a)),
        abilities: project_abilities(card),
        loyalty_abilities: project_loyalty_abilities(card, battlefield),
        loyalty_uses_remaining: card.definition.is_planeswalker().then(|| {
            let allowed: u8 = if card.definition.loyalty_twice_each_turn
                || card.loyalty_twice_this_turn
            {
                2
            } else {
                1
            };
            allowed.saturating_sub(card.loyalty_uses_this_turn)
        }),
        triggered_ability_labels: project_triggered_ability_labels(card),
        static_ability_labels: project_static_ability_labels(card),
        activated_ability_labels: project_activated_ability_labels(card),
        has_stun_counters: card.counter_count(crate::card::CounterType::Stun) > 0,
        wont_untap: state.untap_prevented_by_static(card.id),
        has_finality_counters: card.counter_count(crate::card::CounterType::Finality) > 0,
        dies_to_exile: card.definition.dies_to_exile,
        has_shield_counters: card.counter_count(crate::card::CounterType::Shield) > 0,
        has_prevention_shield,
        doomed_next_damage,
        goaded: !card.goaded_by.is_empty(),
        monstrous: card.monstrous,
        suspected: card.suspected,
        renowned: card.renowned,
        case_solved: card.definition.case.is_some().then_some(card.case_solved),
        class_level: card.definition.is_class().then_some(card.class_level),
        detained: card.detained_by.is_some(),
        untap_locked: card.untap_locked_by.is_some(),
        impending_counters: {
            let n = card.counter_count(crate::card::CounterType::Time);
            let is_impending = card
                .definition
                .keywords
                .iter()
                .any(|k| matches!(k, crate::card::Keyword::Impending(_)));
            (is_impending && n > 0).then_some(n)
        },
        squad_count: (card.squad_count > 0).then_some(card.squad_count),
        pt_modified: {
            let cp_power = cp.map(|c| c.power).unwrap_or_else(|| card.power());
            let cp_toughness = cp.map(|c| c.toughness).unwrap_or_else(|| card.toughness());
            // Creatures and Vehicles (CR 208.3 noncreature P/T — a `*`-power
            // Vehicle like Lumbering Worldwagon shifts with the board) both
            // carry a P/T box; flag it when the live value differs from base.
            // Use the *computed* type so a permanent animated into a creature
            // (Gideon Blackblade during your turn, Awakening of Vitu-Ghazi's
            // land, manlands) shows its P/T box too.
            let live_creature = cp
                .map(|c| c.card_types.contains(&crate::card::CardType::Creature))
                .unwrap_or_else(|| card.definition.is_creature());
            let has_pt_box = live_creature
                || card.definition.subtypes.artifact_subtypes
                    .contains(&crate::card::ArtifactSubtype::Vehicle);
            has_pt_box
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
            match kw {
                crate::card::Keyword::Ward(crate::card::WardCost::Mana(c))
                | crate::card::Keyword::Ward(crate::card::WardCost::ManaAndLife(c, _)) => {
                    Some(c.cmc())
                }
                _ => None,
            }
        }).unwrap_or(0),
        ward_label: card.definition.keywords.iter().find_map(|kw| {
            use crate::card::WardCost as W;
            let crate::card::Keyword::Ward(w) = kw else { return None };
            Some(match w {
                // Plain generic-mana Ward is already carried by `ward_cost`.
                W::Mana(_) => return None,
                W::ManaAndLife(c, n) => format!("Ward—{{{}}}, pay {n} life", c.cmc()),
                W::Life(n) => format!("Ward—pay {n} life"),
                W::Discard(n) => format!("Ward—discard {n}"),
                W::DiscardHand => "Ward—discard your hand".to_string(),
                W::Blight(n) => format!("Ward—Blight {n}"),
                W::CollectEvidence(n) => format!("Ward—Collect evidence {n}"),
                W::SacrificeCreature => "Ward—sacrifice a creature".to_string(),
                W::SacrificePermanents(n) => format!("Ward—sacrifice {n} permanents"),
                W::GenericSourcePower => "Ward—{X} (this creature's power)".to_string(),
                W::LifeSourcePower => "Ward—pay life equal to this creature's power".to_string(),
            })
        }).unwrap_or_default(),
        mana_value: card.definition.cost.cmc(),
        // Computed supertypes so a continuous Legendary grant (Leyline of
        // Singularity, the Ring's emblem) surfaces in the client, matching the
        // legend-rule SBA (CR 704.5j / 613.1c).
        is_legendary: cp
            .map(|c| c.supertypes.contains(&crate::card::Supertype::Legendary))
            .unwrap_or_else(|| {
                card.definition.supertypes.contains(&crate::card::Supertype::Legendary)
            }),
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
        crew_power_bonus: crew_saddle_power_bonus_in(card.id, battlefield),
        saddled: card.saddled,
        crewed_count: card.crewed_by.len() as u32,
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
        chosen_creature_type: card.chosen_creature_type.map(|ct| format!("{ct:?}")),
        // CR 614 — the Siege-cycle mode label, read from the definition's
        // `enter_modes` by the recorded index.
        chosen_mode_label: card.chosen_mode.and_then(|i| {
            card.definition
                .enter_modes
                .as_ref()
                .and_then(|m| m.get(i as usize))
                .map(|m| m.label.to_string())
        }),
        // Auras / Equipment / Fortifications attached to this permanent.
        attachments: battlefield
            .iter()
            .filter(|o| o.attached_to == Some(card.id))
            .map(|o| o.definition.name.to_string())
            .collect(),
        // CR 301.5 / 303 — the host this permanent is attached to, by name.
        attached_to_name: card.attached_to.and_then(|host| {
            battlefield
                .iter()
                .find(|o| o.id == host)
                .map(|o| o.definition.name.to_string())
        }),
        // CR 702.95 — Soulbond partner (only while still on the battlefield).
        soulbond_partner: card
            .soulbond_partner
            .filter(|p| battlefield.iter().any(|o| o.id == *p)),
        saga_final_chapter: card
            .definition
            .saga_chapters
            .iter()
            .map(|(n, _)| *n)
            .max(),
        // CR 712 — DFC / transform UI hints. A transformed permanent's active
        // definition is the back face, so it always carries a `front_face`.
        has_other_face: card.definition.back_face.is_some() || card.front_face.is_some(),
        transformed: card.transformed,
        // CR 708 — face-down permanents render as a 2/2 card back; only the
        // controller may peek at the real card's identity (708.2).
        face_down: card.face_down && card.face_up_def.is_some(),
        face_down_name: (card.face_down && card.controller == viewer_seat)
            .then(|| card.face_up_def.as_ref().map(|d| d.name.to_string()))
            .flatten(),
        // SOS Prepare — surface the inset spell so the client can offer
        // "Cast <name> {cost}" on a prepared creature.
        prepare_spell_name: card
            .definition
            .prepare_spell
            .as_ref()
            .map(|p| p.name.to_string()),
        prepare_cost_label: card
            .definition
            .prepare_spell
            .as_ref()
            .map(|p| format_mana_cost_for_label(&p.cost))
            .unwrap_or_default(),
        prepare_needs_target: card
            .definition
            .prepare_spell
            .as_ref()
            .is_some_and(|p| cursor_needs_target(&p.effect)),
        creature_subtypes: cp
            .map(|c| c.subtypes.creature_types.clone())
            .unwrap_or_else(|| card.definition.subtypes.creature_types.clone()),
        lost_all_abilities: cp.is_some_and(|c| c.lost_all_abilities),
        colors: cp.map(|c| c.colors.clone()).unwrap_or_else(|| {
            let mut cs = card.definition.cost.colors();
            for c in &card.definition.color_indicator {
                if !cs.contains(c) {
                    cs.push(*c);
                }
            }
            cs
        }),
        // CR 700.9 — mirrors the engine's `R::IsModified` (eval.rs): counters,
        // an attached Equipment, or an Aura the controller controls.
        modified: !card.counters.is_empty()
            || battlefield.iter().any(|o| {
                o.attached_to == Some(card.id)
                    && (o.definition.is_artifact()
                        || (o.definition.is_enchantment() && o.controller == card.controller))
            }),
    }
}

/// Project the printed `StaticAbility.description` strings as a flat
/// `Vec<String>` for the client tooltip. Cards without static
/// abilities yield an empty vector. The descriptions are 'static and
/// stable across recomputes — they're the printed Oracle wording.
fn project_static_ability_labels(card: &CardInstance) -> Vec<String> {
    let mut out: Vec<String> = card
        .definition
        .static_abilities
        .iter()
        .map(|s| s.description.to_string())
        .filter(|d| !d.is_empty())
        .collect();
    // Activated abilities an Equipment grants to the creature it's attached to
    // (CR 702.6e — `EquipBonus.activated_abilities`, Wrench's "{3}, {T}: Tap
    // target creature"). Surface them on the Equipment's own tooltip so the
    // grant is visible; they activate off the equipped creature by index.
    if let Some(bonus) = &card.definition.equipped_bonus {
        for a in &bonus.activated_abilities {
            out.push(format!(
                "Equipped: {}: {}",
                ability_cost_label(a),
                ability_effect_label(&a.effect)
            ));
        }
    }
    // CR 603.8 steal-penalty (Bronze Bombshell) lives on a top-level field, not
    // a static ability, so surface it explicitly for the tooltip.
    if let Some(dmg) = card.definition.sacrifice_and_burn_when_stolen {
        out.push(format!(
            "If a player other than its owner controls this, they sacrifice it and it deals {dmg} damage to them"
        ));
    }
    out
}

/// Generate one-line "cost: effect" summaries per activated ability for the
/// client tooltip (the activated analogue of `project_triggered_ability_labels`),
/// so a hover shows "{2}{T}: Draw a card" without opening the detail panel.
/// Skips plain mana abilities (their effect is already the mana line).
fn project_activated_ability_labels(card: &CardInstance) -> Vec<String> {
    card.definition
        .activated_abilities
        .iter()
        .filter(|a| !matches!(a.effect, Effect::AddMana { .. }))
        .map(|a| {
            let cost = ability_cost_label(a);
            let eff = ability_effect_label(&a.effect);
            if cost.is_empty() {
                eff.to_string()
            } else {
                format!("{cost}: {eff}")
            }
        })
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
enum SpellCastKind {
    InstantOrSorcery,
    Creature,
    Other,
}

/// Classify a "whenever you cast a spell" trigger by the card-type gate in its
/// filter, so the client chip reads "Magecraft" only for the instant/sorcery
/// gate and "Creature cast" for a creature gate (Halcyon Glaze).
fn spellcast_filter_kind(filter: Option<&crate::card::Predicate>) -> SpellCastKind {
    use crate::card::{CardType, Predicate, SelectionRequirement as R};
    fn req_mentions(r: &R, ct: &CardType) -> bool {
        match r {
            R::HasCardType(c) => c == ct,
            R::Creature if matches!(ct, CardType::Creature) => true,
            R::And(a, b) | R::Or(a, b) => req_mentions(a, ct) || req_mentions(b, ct),
            _ => false,
        }
    }
    fn pred_kind(p: &Predicate) -> SpellCastKind {
        match p {
            Predicate::EntityMatches { filter, .. } => {
                if req_mentions(filter, &CardType::Instant)
                    || req_mentions(filter, &CardType::Sorcery)
                {
                    SpellCastKind::InstantOrSorcery
                } else if req_mentions(filter, &CardType::Creature) {
                    SpellCastKind::Creature
                } else {
                    SpellCastKind::Other
                }
            }
            Predicate::All(parts) => parts
                .iter()
                .map(pred_kind)
                .find(|k| !matches!(k, SpellCastKind::Other))
                .unwrap_or(SpellCastKind::Other),
            _ => SpellCastKind::Other,
        }
    }
    filter.map(pred_kind).unwrap_or(SpellCastKind::Other)
}

fn trigger_event_label(event: &crate::card::EventSpec) -> &'static str {
    use crate::card::{EventKind, EventScope};
    // "Whenever you cast a [kind] spell" — distinguish magecraft (instant or
    // sorcery) from a creature-cast trigger (Halcyon Glaze) by inspecting the
    // filter's card-type gate instead of assuming any filter means magecraft.
    if matches!(event.kind, EventKind::SpellCast)
        && matches!(event.scope, EventScope::YourControl)
    {
        return match spellcast_filter_kind(event.filter.as_ref()) {
            SpellCastKind::InstantOrSorcery => "Magecraft",
            SpellCastKind::Creature => "Creature cast",
            SpellCastKind::Other => "Spell cast",
        };
    }
    match (&event.kind, event.scope) {
        (EventKind::EntersBattlefield, EventScope::SelfSource) => "ETB",
        (EventKind::EntersBattlefield, EventScope::AnotherOfYours) => "Another ETB",
        (EventKind::EntersBattlefield, EventScope::AnyPlayer) => "Any ETB",
        (EventKind::CreatureDied, EventScope::SelfSource) => "Dies",
        (EventKind::CreatureDied, EventScope::AnotherOfYours) => "Other dies",
        (EventKind::CreatureDied, EventScope::YourControl) => "Your creature dies",
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
        (EventKind::DealsCombatDamageToPlaneswalker, EventScope::SelfSource) => "Combat dmg to PW",
        (EventKind::DealsCombatDamageToPlaneswalker, EventScope::YourControl) => "Your combat dmg to PW",
        (EventKind::ControllerDealtCombatDamage, EventScope::SelfSource) => "You're hit",
        (EventKind::DealsCombatDamageToCreature, EventScope::SelfSource) => "Combat dmg to crea",
        (EventKind::YourInstantOrSorceryDealtDamage, _) => "Your spell deals dmg",
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
        // CR 509 — a creature *you control* becomes blocked (the attacker side,
        // e.g. Tattered Ratter's "whenever a Rat you control becomes blocked").
        (EventKind::BecomesBlocked, EventScope::YourControl) => "Yours blocked",
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
        // "Put into a graveyard from the battlefield" observers (CR 700.4) —
        // used by equip-granted death watchers (Tarrian's Soulcleaver) and
        // graveyard-matters payoffs.
        (EventKind::PutIntoGraveyard, EventScope::SelfSource) => "Put into GY",
        (EventKind::PutIntoGraveyard, EventScope::YourControl) => "Yours to GY",
        (EventKind::PutIntoGraveyard, EventScope::OpponentControl) => "Opp to GY",
        (EventKind::PutIntoGraveyard, EventScope::AnyPlayer) => "Any to GY",
        (EventKind::LandPutIntoGraveyard, _) => "Land to GY",
        // Enrage (CR 702.130) — "Whenever this creature is dealt damage."
        (EventKind::DealtDamage, EventScope::SelfSource) => "Enrage",
        (EventKind::DealtDamage, EventScope::YourControl) => "Your crea dealt dmg",
        (EventKind::DealtDamage, EventScope::AnyPlayer) => "Any crea dealt dmg",
        // Tap-matters triggers. `YouTapped` is the "whenever you tap …" scope
        // (Sharae, Solitary Sanctuary); the others key off the tapped
        // permanent's controller (Magda-style).
        (EventKind::Tapped, EventScope::YouTapped) => "You tap",
        (EventKind::Tapped, EventScope::SelfSource) => "When tapped",
        (EventKind::Tapped, EventScope::YourControl) => "Yours tapped",
        (EventKind::Tapped, EventScope::OpponentControl) => "Enemy tapped",
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

/// Whether the client should arm the in-scene targeting cursor before
/// submitting this effect: it takes a target AND that target lives on the
/// board. Slot-0 targets in an off-board zone (graveyard / exile — "return
/// target card from your graveyard") are gathered by an engine-side
/// `ChooseCards` suspend instead, so the client must submit with no target.
fn cursor_needs_target(effect: &crate::effect::Effect) -> bool {
    effect.requires_target()
        && !effect
            .target_filter_for_slot(0)
            .is_some_and(|f| f.mentions_offboard_zone())
}

fn project_loyalty_abilities(
    card: &CardInstance,
    battlefield: &[CardInstance],
) -> Vec<crate::net::LoyaltyAbilityView> {
    // Printed + statics-granted (Kasmina / Ichormoon) — the same list the
    // activation path accepts, so granted indices are clickable.
    crate::game::effective_loyalty_abilities(card, battlefield)
        .iter()
        .enumerate()
        .map(|(i, a)| crate::net::LoyaltyAbilityView {
            index: i,
            loyalty_cost: a.loyalty_cost,
            x_cost: a.x_cost,
            effect_label: ability_effect_label(&a.effect).to_string(),
            needs_target: cursor_needs_target(&a.effect),
        })
        .collect()
}

fn project_abilities(card: &CardInstance) -> Vec<AbilityView> {
    card.definition
        .activated_abilities
        .iter()
        // Instance-granted abilities (Urza's Saga chapters) surface after
        // the printed ones — same index order `activate_ability` resolves.
        .chain(card.granted_activated_abilities.iter())
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
                needs_target: cursor_needs_target(&a.effect),
                is_mana: is_mana_ability(&a.effect),
                once_per_turn_used: a.once_per_turn && card.once_per_turn_used.contains(&i),
                gate_label,
                gate_blocked: false,
                opponents_only: a.opponents_only,
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
        Predicate::CastSpellSharesChosenColorOfSource => "cast spell of chosen color".into(),
        Predicate::CastSpellTargetsMatch(_) => "cast spell targets match".into(),
        Predicate::CastSpellIsAdventure => "after Adventure cast".into(),
        Predicate::CastSpellMatches(_) => "cast spell matches".into(),
        Predicate::CastSpellManaSpentAtLeast(n) => format!("if ≥{n} mana spent"),
        Predicate::IncrementSatisfied => "Increment (mana > P or T)".into(),
        Predicate::CommittedCrimeThisTurn { .. } => "if you committed a crime".into(),
        Predicate::ControlsOutlaw { .. } => "if you control an outlaw".into(),
        Predicate::SacrificedWasOutlaw => "if an outlaw was sacrificed".into(),
        Predicate::SacrificedWasArtifact => "if an artifact was sacrificed".into(),
        Predicate::SacrificedWasVehicle => "if a Vehicle was sacrificed".into(),
        Predicate::SourceSaddled => "while saddled".into(),
        Predicate::OpponentControlsMoreLandsThanYou => "if behind on lands".into(),
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
            ManaSymbol::PhyrexianHybrid(a, b) => format!("{{{a}/{b}/P}}"),
            ManaSymbol::MonoHybrid(n, c) => format!("{{{n}/{c}}}"),
            ManaSymbol::Snow => "{S}".into(),
            ManaSymbol::X => "{X}".into(),
        };
        parts.push(tok);
    }
    if ability.tap_cost {
        parts.push("{T}".into());
    }
    // {E} (energy) cost — Aether Hub's `{T}, Pay {E}`, Servant of the Conduit.
    // Rendered as one `{E}` per energy so the tooltip mirrors card text.
    if ability.energy_cost > 0 {
        parts.push("{E}".repeat(ability.energy_cost as usize));
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
    // Tap-N-permanents activations (Heritage Druid's "Tap three untapped Elves
    // you control") — `tap_n_filter`.
    if let Some((req, n)) = ability.tap_n_filter.as_ref() {
        parts.push(format!("Tap {n} {}s", requirement_noun(req)));
    }
    // Return-another-to-hand activations (Quirion Ranger, Wirewood Symbiote)
    // — `bounce_other_filter`.
    if let Some((req, n)) = ability.bounce_other_filter.as_ref() {
        let noun = requirement_noun(req);
        if *n == 1 {
            parts.push(format!("Return a {noun}"));
        } else {
            parts.push(format!("Return {n} {noun}s"));
        }
    }
    // Discard-as-cost (Fauna Shaman, Survival of the Fittest) — `discard_cost`.
    if let Some((req, n)) = ability.discard_cost.as_ref() {
        let noun = requirement_noun(req);
        if *n == 1 {
            parts.push(format!("Discard a {noun}"));
        } else {
            parts.push(format!("Discard {n} {noun}s"));
        }
    }
    // Remove-counter-as-cost (Walking Ballista, Triskelion, Hangarback Walker)
    // — `remove_counter_cost`. The counter kind is rendered with a short label.
    if let Some((kind, n)) = ability.remove_counter_cost.as_ref() {
        let label = counter_kind_label(kind);
        let sep = if label.is_empty() { "" } else { " " };
        if *n == 1 {
            parts.push(format!("Remove a{sep}{label} counter"));
        } else {
            parts.push(format!("Remove {n}{sep}{label} counters"));
        }
    }
    // Return-self-as-cost (Grinning Ignus, Rootha) — "Return this to hand"
    // so the tooltip shows the bounce rider rather than looking free for
    // mana alone.
    if ability.return_self_cost {
        parts.push("Return this to hand".into());
    }
    // Discard-this-as-cost (Elemental Masterpiece) — the from-hand
    // "Discard this card:" cost line.
    if ability.discard_self_cost {
        parts.push("Discard this".into());
    }
    // Collect-evidence-as-cost (Forensic Researcher) — CR 701.59.
    if let Some(n) = ability.collect_evidence_cost {
        parts.push(format!("Collect evidence {n}"));
    }
    let mut label = if parts.is_empty() { "0".into() } else { parts.join(", ") };
    // Opponent-only escape clauses (Detention Vortex) — flag who may activate
    // so the tooltip doesn't read as a self-usable ability.
    if ability.opponents_only {
        label.push_str(" (opponents only)");
    }
    label
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

/// Short label for a counter kind used in "Remove a [label] counter" cost
/// riders. Falls back to a generic "counter" wording for rarer kinds.
fn counter_kind_label(kind: &crate::card::CounterType) -> &'static str {
    use crate::card::CounterType as C;
    match kind {
        C::PlusOnePlusOne => "+1/+1",
        C::MinusOneMinusOne => "-1/-1",
        C::Charge => "charge",
        C::Loyalty => "loyalty",
        _ => "",
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
        // "You may pay {X}" (Well of Lost Dreams) — surface the paid-for body.
        Effect::MayPayGenericUpTo { body, .. } => ability_effect_label(body),
        // Reflexive / optional-cost wrappers surface the payoff they gate.
        Effect::Reflexive { body } | Effect::MayPayX { body, .. } => ability_effect_label(body),
        Effect::MayDiscard { then, .. }
        | Effect::MayTap { then, .. }
        | Effect::MaySacrifice { then, .. }
        | Effect::MaySacrificeSource { then, .. } => ability_effect_label(then),
        Effect::Learn { .. } => "Learn",
        Effect::Venture => "Venture into the dungeon",
        Effect::Populate { .. } => "Populate",
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
        Effect::RemoveAnyCounter { .. } => "Remove a counter",
        Effect::RemoveCountersUpTo { .. } => "Remove counters",
        Effect::ExileReturnNextEndStep { .. } | Effect::ExileReturnToOwnerNextEndStep { .. } => {
            "Flicker until end of turn"
        }
        Effect::CreateToken { .. } => "Create token",
        Effect::CreateTokenAttachedTo { .. } | Effect::CreateTokenAttachedToEach { .. } => {
            "Create attached token"
        }
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
        Effect::SetSaddled { .. } => "Saddle",
        Effect::Untap { .. } => "Untap",
        Effect::PumpPT { .. } => "Pump",
        Effect::SetBasePT { .. } => "Set base P/T",
        Effect::SwitchPT { .. } => "Switch P/T",
        Effect::Process { then, .. } => {
            // Surface the rider's label — the "process from exile" step
            // resolves through the decision panel.
            let inner = ability_effect_label(then);
            if inner == "Activate" { "Process" } else { inner }
        }
        Effect::GrantKeyword { .. } => "Grant keyword",
        Effect::GrantKeywords { .. } => "Grant keywords",
        Effect::AddPoison { .. } => "Add poison",
        Effect::RevealUntilFind { .. } => "Reveal until find",
        Effect::AddFirstSpellTax { .. } => "Cost tax",
        Effect::Drain { .. } => "Drain",
        Effect::SetNoMaxHandSize { .. } => "No max hand size",
        Effect::FlipCoin { .. } => "Flip coin",
        Effect::Proliferate => "Proliferate",
        Effect::LookAtTop { .. } => "Look at top",
        Effect::LookTopMayRevealMatchToHandElseBottom { .. } => "Look at top (draw or bottom)",
        Effect::CommandTheDreadhorde => "Reanimate from graveyards",
        Effect::RearrangeTop { .. } => "Rearrange top",
        Effect::ShuffleGraveyardIntoLibrary { .. } => "Shuffle into library",
        Effect::PutOnLibraryFromHand { .. } => "Put on library",
        Effect::RevealTopAndDrawIf { .. } => "Reveal top",
        Effect::CopySpell { .. } => "Copy spell",
        Effect::CopySpellWithRiders { .. } => "Copy spell (haste, sac at end step)",
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
        Effect::PreventCombatDamageByTargetThisTurn { .. } => "Prevent combat damage by target",
        Effect::CantBlockSourceThisTurn { .. } => "Target can't block this",
        Effect::SkipTurns { .. } => "Skip turns",
        Effect::SetLifeTotal { .. } => "Set life total",
        Effect::ExchangeLifeTotals { .. } => "Exchange life totals",
        Effect::PreventNextDamage { .. } => "Prevent damage",
        Effect::PreventNextDamageAndGainLife { .. } => "Prevent damage, gain life",
        Effect::PreventAllDamageThisTurn { .. } => "Prevent all damage",
        Effect::ReplaceNextDamageWithDestroy { .. } => "Destroy on next damage",
        Effect::DamageCantBePreventedThisTurn => "Damage can't be prevented",
        Effect::LifeGainLockThisTurn { .. } => "Lock lifegain",
        Effect::GrantSpellsUncounterableThisTurn { .. } => "Spells can't be countered",
        Effect::GrantHexproofFromColorThisTurn { .. } => "Hexproof from color",
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
        Effect::MayExileFromYourGraveyard { .. } => "Exile from your graveyard",
        Effect::ExileAllGraveyards { .. } => "Exile all graveyards",
        Effect::CreateTokenAttacking { .. } => "Create attacking tokens",
        Effect::Amass { .. } => "Amass",
        Effect::Myriad => "Myriad",
        Effect::Enlist => "Enlist",
        Effect::GrantNextInstantOrSorceryDiscountThisTurn { .. } => "Discount next spell",
        Effect::SupportCounters { .. } => "Support",
        Effect::Detain { .. } => "Detain",
        Effect::Fateseal { .. } => "Fateseal",
        Effect::Discover { .. } => "Discover",
        Effect::Cascade { .. } => "Cascade",
        Effect::CollectEvidence { .. } => "Collect evidence",
        Effect::CollectEvidenceX { .. } => "Collect evidence X",
        Effect::Forage { .. } => "Forage",
        Effect::Endure { .. } => "Endure",
        Effect::DigToHandLoseLife { .. } => "Dig, lose life per card kept",
        Effect::Suspect { .. } => "Suspect",
        Effect::Ascend { .. } => "Ascend",
        Effect::ReturnSelfTappedWithCounters { .. } => "Return tapped with counters",
        Effect::ReturnTopCreatureFromGraveyard { .. } => "Reanimate top creature",
        Effect::ChooseRandomGraveyardCardCreatureToBattlefieldElseHand { .. } => {
            "Random GY card to play/hand"
        }
        Effect::Regenerate { .. } => "Regenerate",
        Effect::SacrificePermanent { .. } => "Sacrifice",
        Effect::LoseKeywordThisTurn { .. } => "Remove keyword",
        Effect::AddManaEqualToPermanentCost { .. } => "Add mana of cost",
        Effect::NameCardExileMatchingAllZones => "Name & exile all copies",
        Effect::ChooseTypeRevealTopPartition { .. } => "Reveal & sort by type",
        Effect::FertileImagination { .. } => "Saprolings per type",
        Effect::GuildFeud => "Duel top creatures",
        Effect::AethermagesTouch { .. } => "Flash in a creature",
        Effect::InfernalTutor => "Tutor",
        Effect::IgnorantBliss => "Blink your hand",
        Effect::Dovescape => "Counter → Birds",
        Effect::IsperiaReveal => "Name → tutor flyer",
        Effect::GraveBetrayalRegister | Effect::GraveBetrayalReanimate => "Steal the dead",
        Effect::KindleTheCarnage => "Discard → board burn",
        Effect::ChooseTwoColorsForSource => "Choose two colors",
        Effect::GainLifePerChosenColorOfCast => "Guild lifegain",
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
    fn no_choice_payload(pool: &ManaPayload) -> bool {
        match pool {
            // No-choice payloads — auto-tap activates them on the user's
            // behalf without surfacing a menu entry.
            ManaPayload::Colors(_) | ManaPayload::Colorless(_) | ManaPayload::OfColor(_, _) => true,
            // Spend-restricted mana (Omen Hawker, the Strixhaven school
            // dorks) is still a fixed-output mana ability — recurse past the
            // restriction wrapper.
            ManaPayload::Restricted(inner, _)
            | ManaPayload::RestrictedToChosenType(inner)
            | ManaPayload::RestrictedToChosenTypePlain(inner) => no_choice_payload(inner),
            _ => false,
        }
    }
    match effect {
        Effect::AddMana { pool, .. } => no_choice_payload(pool),
        Effect::Seq(steps) => !steps.is_empty() && steps.iter().all(is_mana_ability),
        // Conditional fixed-output mana (Ilysian Caryatid, Raucous Audience):
        // a mana source whenever both branches are no-choice mana abilities.
        Effect::If { then, else_, .. } => is_mana_ability(then) && is_mana_ability(else_),
        _ => false,
    }
}

fn project_stack(item: &StackItem, state: &GameState, viewer_seat: usize) -> StackItemView {
    match item {
        StackItem::Spell { card, caster, target, additional_targets, .. } => {
            // CR 708.1 — a face-down spell reveals only to its caster.
            // Opponents and spectators get the `Hidden` view; the caster sees
            // the real name stashed in `face_up_def` (the live definition is
            // already the nameless 2/2 for Morph casts).
            if card.face_down && *caster != viewer_seat {
                return StackItemView::Hidden { source: card.id, controller: *caster };
            }
            let name = if card.face_down {
                card.face_up_def
                    .as_ref()
                    .map(|d| d.name.to_string())
                    .unwrap_or_else(|| card.definition.name.to_string())
            } else {
                card.definition.name.to_string()
            };
            StackItemView::Known(KnownStackItem {
                source: card.id,
                controller: *caster,
                name,
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
    use crate::game::TriggerPush;
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
        g.remove_from_battlefield_to_graveyard_raw(a);
        g.remove_from_battlefield_to_graveyard_raw(b);
        let view = project(&g, 0);
        assert_eq!(view.permanents_to_graveyard_this_turn, 2);
    }

    #[test]
    fn project_surfaces_wont_untap() {
        // A creature enchanted by Plumes of Peace (a `PreventUntap` static)
        // reads as `wont_untap`; an unencumbered one does not.
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let free = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let aura = g.add_card_to_battlefield(0, catalog::plumes_of_peace());
        g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
        let v = project(&g, 0);
        assert!(v.battlefield.iter().find(|p| p.id == bear).unwrap().wont_untap, "locked");
        assert!(!v.battlefield.iter().find(|p| p.id == free).unwrap().wont_untap, "free");
    }

    #[test]
    fn project_surfaces_void_active() {
        // EOE Void — the view flags a seat whose Void condition is met so the
        // client can show a "✦ Void" chip.
        let mut g = two_player_game();
        assert!(!project(&g, 0).players[0].void_active, "dormant by default");
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.remove_from_battlefield_to_graveyard_raw(bear); // a nonland permanent left
        assert!(project(&g, 0).players[0].void_active, "nonland leaving the battlefield → Void");
    }

    #[test]
    fn project_surfaces_experience() {
        // The view exposes each seat's experience-counter count so the client
        // can badge an experience chip (Mizzix/Ezuri decks).
        let mut g = two_player_game();
        assert_eq!(project(&g, 0).players[0].experience, 0, "none by default");
        g.players[0].experience = 3;
        assert_eq!(project(&g, 0).players[0].experience, 3, "surfaced to the view");
    }

    #[test]
    fn project_surfaces_at_max_speed() {
        // CR 702.179c — the view pre-derives the max-speed flag so the client
        // can highlight live "Max speed —" abilities.
        let mut g = two_player_game();
        g.players[0].speed = 3;
        assert!(!project(&g, 0).players[0].at_max_speed, "speed 3 is not max");
        g.players[0].speed = 4;
        assert!(project(&g, 0).players[0].at_max_speed, "speed 4 is max speed");
    }

    #[test]
    fn spectator_view_hides_every_hand_and_marks_sentinel_seat() {
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::plains());
        g.add_card_to_hand(1, catalog::plains());

        // A seated player sees their own hand as Known.
        let seat0 = project(&g, 0);
        assert!(matches!(seat0.players[0].hand[0], HandCardView::Known(_)));
        assert!(matches!(seat0.players[1].hand[0], HandCardView::Hidden { .. }));

        // A spectator sees no seat and every hand hidden.
        let spec = project_spectator(&g);
        assert_eq!(spec.your_seat, crate::net::SPECTATOR_SEAT);
        assert!(
            spec.players.iter().all(|p| p
                .hand
                .iter()
                .all(|c| matches!(c, HandCardView::Hidden { .. }))),
            "spectator must not see any player's hand contents",
        );
        // Public board state is still visible (life totals project through).
        assert_eq!(spec.players.len(), 2);
        // No cast/attack/block affordances for a non-seated viewer.
        assert!(spec.castable_hand.is_empty());
        assert!(spec.legal_attackers.is_empty());
        assert!(spec.legal_blockers.is_empty());
    }

    #[test]
    fn spectator_sees_a_decision_is_pending_but_not_its_contents() {
        use crate::decision::Decision;
        let mut g = two_player_game();
        g.add_card_to_hand(0, catalog::plains());
        // Install a discard decision for seat 0.
        let hand: Vec<(crate::card::CardId, String)> = g.players[0]
            .hand
            .iter()
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        g.pending_decision = Some(crate::game::types::PendingDecision {
            decision: Decision::Discard { player: 0, count: 1, hand },
            resume: crate::game::types::ResumeContext::CleanupDiscard { player: 0 },
        });

        let spec = project_spectator(&g);
        let pd = spec.pending_decision.expect("spectator sees a decision is pending");
        assert_eq!(pd.acting_player, 0);
        assert!(pd.decision.is_none(), "spectator must not see decision contents");
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
            mint_mites_for: None,
            target: PreventionTarget::Player(0),
            destroy: false,
            remaining: None,
            gain_life: false,
            source: None,
            one_event: false,
            reflect: false,
            source_controller: None,
            redirect_to: None,
        });
        state.prevention_shields.push(PreventionShield {
            mint_mites_for: None,
            target: PreventionTarget::Permanent(bear),
            destroy: false,
            remaining: Some(2),
            gain_life: false,
            source: None,
            one_event: false,
            reflect: false,
            source_controller: None,
            redirect_to: None,
        });
        // A Kill-Suit Cultist "destroy on next damage" shield on a second
        // creature reads as `doomed_next_damage`, NOT as protection.
        let doomed = state.add_card_to_battlefield(1, catalog::grizzly_bears());
        state.prevention_shields.push(PreventionShield {
            mint_mites_for: None,
            target: PreventionTarget::Permanent(doomed),
            destroy: true,
            remaining: None,
            gain_life: false,
            source: None,
            one_event: true,
            reflect: false,
            source_controller: None,
            redirect_to: None,
        });
        state.damage_cant_be_prevented_this_turn = true;
        let v = project(&state, 0);
        assert!(v.players[0].has_prevention_shield, "P0 is shielded");
        assert!(!v.players[1].has_prevention_shield, "P1 is not");
        let bear_v = v.battlefield.iter().find(|p| p.id == bear).unwrap();
        assert!(bear_v.has_prevention_shield, "protective shield surfaces");
        assert!(!bear_v.doomed_next_damage);
        let doomed_v = v.battlefield.iter().find(|p| p.id == doomed).unwrap();
        assert!(doomed_v.doomed_next_damage, "destroy shield reads as doomed");
        assert!(!doomed_v.has_prevention_shield, "a destroy shield is not protection");
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
    fn continuous_legendary_grant_surfaces_in_the_view() {
        let mut state = two_player_game();
        let bear = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        // A vanilla creature isn't legendary...
        assert!(!project(&state, 0).battlefield.iter().find(|p| p.id == bear).unwrap().is_legendary);
        // ...until Leyline of Singularity grants the supertype (CR 704.5j).
        state.add_card_to_battlefield(0, catalog::leyline_of_singularity());
        assert!(project(&state, 0).battlefield.iter().find(|p| p.id == bear).unwrap().is_legendary);
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
    fn lifegain_lock_surfaces_in_the_view() {
        let mut state = two_player_game();
        // Sunspine Lynx locks lifegain for every player (CR 119.7).
        state.add_card_to_battlefield(0, catalog::sunspine_lynx());
        let v = project(&state, 0);
        assert!(v.players[0].cannot_gain_life, "controller's lifegain is locked");
        assert!(v.players[1].cannot_gain_life, "opponent's lifegain is locked too");
    }

    #[test]
    fn life_lock_surfaces_in_the_view() {
        let mut state = two_player_game();
        assert!(!project(&state, 0).players[0].life_locked, "not locked by default");
        state.players[0].life_locked_this_turn = true;
        let v = project(&state, 0);
        assert!(v.players[0].life_locked, "controller's frozen life total surfaces");
        assert!(!v.players[1].life_locked, "opponent's life total is not frozen");
    }

    #[test]
    fn player_hexproof_surfaces_in_the_view() {
        let mut state = two_player_game();
        // Aegis of the Gods grants its controller hexproof (CR 702.11).
        state.add_card_to_battlefield(0, catalog::aegis_of_the_gods());
        let v = project(&state, 0);
        assert!(v.players[0].has_hexproof, "controller's hexproof is surfaced");
        assert!(!v.players[1].has_hexproof, "the opponent has no hexproof");
    }

    #[test]
    fn kaya_static_hides_opponent_hexproof_from_that_viewer() {
        let mut state = two_player_game();
        // Player 1 has hexproof (Aegis); player 0 controls Kaya's ignore-static.
        state.add_card_to_battlefield(1, catalog::aegis_of_the_gods());
        state.add_card_to_battlefield(0, catalog::kaya_bane_of_the_dead());
        // From Kaya's controller (seat 0), the opponent no longer reads hexproof.
        assert!(!project(&state, 0).players[1].has_hexproof, "Kaya's controller ignores it");
        // From the opponent's own seat (seat 1), their hexproof still shows.
        assert!(project(&state, 1).players[1].has_hexproof, "self-view keeps hexproof");
    }

    #[test]
    fn detained_surfaces_in_the_view() {
        let mut state = two_player_game();
        let a = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        state.battlefield_find_mut(a).unwrap().detained_by = Some(1);
        let v = project(&state, 0);
        assert!(v.battlefield.iter().find(|p| p.id == a).unwrap().detained);
        assert!(!v.battlefield.iter().find(|p| p.id == b).unwrap().detained);
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
    fn face_down_permanent_hidden_from_opponent_visible_to_controller() {
        let mut state = two_player_game();
        let top = state.next_id();
        state.players[0].library.insert(
            0,
            crate::card::CardInstance::new(top, catalog::grizzly_bears(), 0),
        );
        let ctx = crate::game::effects::EffectContext::for_ability(top, 0, None);
        let mut events = vec![];
        state.manifest_card(top, 0, &ctx, &mut events);

        // Controller (seat 0) sees the face-down flag and the real name.
        let own = project(&state, 0);
        let mine = own.battlefield.iter().find(|p| p.id == top).unwrap();
        assert!(mine.face_down);
        assert_eq!(mine.name, "", "the public name stays blank");
        assert_eq!(mine.face_down_name.as_deref(), Some("Grizzly Bears"));

        // Opponent (seat 1) sees a nameless 2/2 with no peek.
        let opp = project(&state, 1);
        let theirs = opp.battlefield.iter().find(|p| p.id == top).unwrap();
        assert!(theirs.face_down);
        assert_eq!(theirs.name, "");
        assert!(theirs.face_down_name.is_none(), "opponent can't peek");
    }

    #[test]
    fn face_down_spell_on_stack_hidden_from_opponent_named_for_caster() {
        let mut state = two_player_game();
        let id = state.next_id();
        let mut card = crate::card::CardInstance::new(id, catalog::grizzly_bears(), 0);
        card.turn_face_down();
        state.stack.push(crate::game::StackItem::Spell {
            card: Box::new(card),
            caster: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: 0,
            converged_value: 0,
            mana_spent: 0,
            uncounterable: false,
        });

        // Caster (seat 0) sees the real name of their own face-down spell.
        let own = project(&state, 0);
        match &own.stack[0] {
            StackItemView::Known(k) => assert_eq!(k.name, "Grizzly Bears"),
            other => panic!("caster should see a Known item, got {other:?}"),
        }

        // Opponent (seat 1) and spectators get the Hidden view.
        let opp = project(&state, 1);
        assert!(
            matches!(opp.stack[0], StackItemView::Hidden { source, controller } if source == id && controller == 0),
            "opponent must not see face-down spell identity"
        );
        let spec = super::project_spectator(&state);
        assert!(matches!(spec.stack[0], StackItemView::Hidden { .. }));
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
        g.stack.push(TriggerPush::new(bolt_id, 0, Effect::Noop).build());
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
        state.remove_from_battlefield_to_graveyard_raw(b);
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
    fn face_down_exiled_card_is_masked_from_opponents() {
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let idx = state.battlefield.iter().position(|c| c.id == id).unwrap();
        let mut card = state.battlefield.remove(idx);
        card.face_down = true;
        state.exile.push(card);

        let owner_view = project(&state, 0);
        assert_eq!(owner_view.exile[0].name, "Grizzly Bears", "controller sees the card");
        assert!(owner_view.exile[0].face_down);
        let opp_view = project(&state, 1);
        assert_eq!(opp_view.exile[0].name, "Face-down card", "opponent sees a mask");
        assert_eq!(opp_view.exile[0].mana_value, 0);
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
    fn graveyard_view_surfaces_scavenge_grant() {
        // Varolz grants scavenge to the controller's graveyard creatures; the
        // view advertises the cost only while a granting source is in play.
        let mut state = two_player_game();
        let dead = state.add_card_to_graveyard(0, catalog::grizzly_bears());
        let before = project(&state, 0);
        assert!(
            before.players[0].graveyard.iter().find(|c| c.id == dead).unwrap().scavenge_cost.is_none(),
            "no scavenge without Varolz",
        );
        state.add_card_to_battlefield(0, catalog::varolz_the_scar_striped());
        let after = project(&state, 0);
        assert!(
            after.players[0].graveyard.iter().find(|c| c.id == dead).unwrap().scavenge_cost.is_some(),
            "scavenge cost surfaced under Varolz",
        );
    }

    #[test]
    fn permanent_view_surfaces_squad_count() {
        // A creature cast paying Squad twice reports squad_count = 2; its token
        // copies (and plain casts) report None.
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::vanguard_suppressor());
        state.battlefield_find_mut(id).unwrap().squad_count = 2;
        let perm = project(&state, 0).battlefield.into_iter().find(|p| p.id == id).unwrap();
        assert_eq!(perm.squad_count, Some(2));
        let plain = state.add_card_to_battlefield(0, catalog::grizzly_bears());
        let pv = project(&state, 0).battlefield.into_iter().find(|p| p.id == plain).unwrap();
        assert_eq!(pv.squad_count, None);
    }

    #[test]
    fn permanent_view_surfaces_impending_countdown() {
        // An Overlord with time counters reports its impending countdown and
        // is projected as a non-creature; once the counters are gone it's a
        // creature with no countdown.
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::overlord_of_the_boilerbilges());
        state.battlefield_find_mut(id).unwrap()
            .add_counters(crate::card::CounterType::Time, 4);
        let perm = project(&state, 0).battlefield.into_iter().find(|p| p.id == id).unwrap();
        assert_eq!(perm.impending_counters, Some(4));
        assert!(!perm.card_types.contains(&crate::card::CardType::Creature), "non-creature while counting down");
        state.battlefield_find_mut(id).unwrap()
            .remove_counters(crate::card::CounterType::Time, 4);
        let perm = project(&state, 0).battlefield.into_iter().find(|p| p.id == id).unwrap();
        assert_eq!(perm.impending_counters, None);
        assert!(perm.card_types.contains(&crate::card::CardType::Creature), "creature once counters are gone");
    }

    #[test]
    fn permanent_view_includes_static_ability_labels() {
        // Top of the Class has printed statics ("Prepared creatures you
        // control get +1/+1 / have flying") — the view should surface the
        // description strings in `static_ability_labels`. (The old fixture,
        // Tenured Inkcaster, lost its synthesized anthem when its body was
        // rewritten to the real oracle.)
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::top_of_the_class());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert!(!perm.static_ability_labels.is_empty(),
            "Top of the Class has printed statics — view must surface them");
        assert!(
            perm.static_ability_labels.iter().any(|s| s.contains("Prepared")),
            "static_ability_labels should mention Prepared: {:?}",
            perm.static_ability_labels,
        );
    }

    #[test]
    fn permanent_view_labels_your_control_death_and_etb_triggers() {
        // Vindictive Xborg fires on "another creature you control dies"
        // (CreatureDied/YourControl) and Griffin Protector on "another creature
        // you control enters" (EntersBattlefield/YourControl). Both scopes now
        // carry a human event label rather than a bare effect string.
        let mut state = two_player_game();
        let vamp = state.add_card_to_battlefield(0, catalog::vindictive_vampire());
        let griffin = state.add_card_to_battlefield(0, catalog::griffin_protector());
        let view = project(&state, 0);
        let vp = view.battlefield.iter().find(|p| p.id == vamp).unwrap();
        assert!(vp.triggered_ability_labels.iter().any(|s| s.starts_with("Your creature dies:")),
            "death-of-ally trigger labelled: {:?}", vp.triggered_ability_labels);
        let gp = view.battlefield.iter().find(|p| p.id == griffin).unwrap();
        assert!(gp.triggered_ability_labels.iter().any(|s| s.starts_with("Your ETB:")),
            "ally-ETB trigger labelled: {:?}", gp.triggered_ability_labels);
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
    fn permanent_view_surfaces_activated_ability_labels() {
        // Ral Zarek is a planeswalker (loyalty abilities, not activated) — use a
        // creature with a real activated ability. Zhur-Taa Druid's mana ability is
        // filtered out, but Prodigal Sorcerer's "{T}: 1 damage" should surface.
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::prodigal_sorcerer());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert!(
            perm.activated_ability_labels.iter().any(|s| s.contains("Deal damage")),
            "activated ability should surface: {:?}",
            perm.activated_ability_labels
        );
    }

    #[test]
    fn permanent_view_surfaces_equipment_granted_activated_ability() {
        // Wrench grants "{3}, {T}: Tap target creature" via
        // EquipBonus.activated_abilities — the Equipment's tooltip must show it.
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::wrench());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert!(
            perm.static_ability_labels.iter().any(|s| s.starts_with("Equipped:")),
            "granted activated ability should appear: {:?}",
            perm.static_ability_labels
        );
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
    fn woe_adventure_and_role_labels_are_specific() {
        use crate::card::Predicate;
        // Chancellor of Tales' "after Adventure cast" trigger predicate.
        assert_eq!(predicate_short_label(&Predicate::CastSpellIsAdventure), "after Adventure cast");
        // Asinine Antics / Curse of the Werefox mint an attached Role token —
        // not the generic "Create token" label.
        assert_eq!(
            ability_effect_label(&catalog::curse_of_the_werefox().effect),
            "Create attached token",
        );
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
            ..Default::default()
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
            ..Default::default()
        };
        assert_eq!(ability_cost_label(&ab_x), "{X}",
            "X-cost ability renders as {{X}}");
    }

    #[test]
    fn effect_labels_cover_new_war_effects() {
        use crate::effect::Effect;
        assert_eq!(
            ability_effect_label(&Effect::CommandTheDreadhorde),
            "Reanimate from graveyards",
        );
        assert_eq!(
            ability_effect_label(&Effect::LookTopMayRevealMatchToHandElseBottom {
                filter: crate::card::SelectionRequirement::Creature,
            }),
            "Look at top (draw or bottom)",
        );
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
            ..Default::default()
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
            ..Default::default()
        };
        let label = ability_cost_label(&petal);
        assert!(label.contains("{T}") && label.contains("Sac"),
            "{label} = `{{T}}, Sac`-style for Lotus Petal");
    }

    /// Energy, discard, and remove-counter costs must surface in the tooltip
    /// so abilities like Aether Hub, Fauna Shaman, and Walking Ballista don't
    /// look free for their mana/tap alone.
    #[test]
    fn ability_cost_label_renders_energy_discard_and_counter_riders() {
        use crate::card::{CounterType, SelectionRequirement as R};
        use crate::effect::ActivatedAbility;
        use crate::mana::{cost, generic};
        // Aether Hub: {T}, Pay {E}: Add any color.
        let hub = ActivatedAbility { tap_cost: true, energy_cost: 1, ..Default::default() };
        assert!(ability_cost_label(&hub).contains("{E}"), "energy cost shown");
        // Fauna Shaman: {G}, {T}, Discard a creature card.
        let shaman = ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Creature, 1)),
            ..Default::default()
        };
        assert!(ability_cost_label(&shaman).contains("Discard a creature"), "discard cost shown");
        // Walking Ballista: Remove a +1/+1 counter.
        let ballista = ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            ..Default::default()
        };
        assert_eq!(ability_cost_label(&ballista), "Remove a +1/+1 counter");
        // Forensic Researcher: {T}, Collect evidence 3: ...
        let researcher = ActivatedAbility {
            tap_cost: true,
            collect_evidence_cost: Some(3),
            ..Default::default()
        };
        assert!(
            ability_cost_label(&researcher).contains("Collect evidence 3"),
            "collect-evidence cost shown in the tooltip",
        );
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

        // Heritage Druid: "Tap three untapped Elves you control".
        let heritage = ActivatedAbility {
            tap_n_filter: Some((R::HasCreatureType(crate::card::CreatureType::Elf), 3)),
            ..Default::default()
        };
        assert!(ability_cost_label(&heritage).contains("Tap 3"), "tap-N rider shown");

        // Quirion Ranger / Wirewood Symbiote: "Return a [permanent] you control".
        let quirion = ActivatedAbility {
            bounce_other_filter: Some((R::HasLandType(crate::card::LandType::Forest), 1)),
            ..Default::default()
        };
        assert!(ability_cost_label(&quirion).contains("Return a"), "bounce rider shown");
    }

    /// `return_self_cost` activations (Grinning Ignus, Rootha) must show the
    /// bounce rider so the tooltip doesn't look free for tap+mana alone.
    #[test]
    fn ability_cost_label_renders_return_self_rider() {
        use crate::effect::ActivatedAbility;
        use crate::mana::{cost, r};
        let ignus = ActivatedAbility {
            mana_cost: cost(&[r()]),
            return_self_cost: true,
            ..Default::default()
        };
        let label = ability_cost_label(&ignus);
        assert!(label.contains("{R}"), "got: {label}");
        assert!(label.contains("Return this to hand"), "bounce rider shown: {label}");
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

    /// Omen Hawker taps for spend-restricted `{C}{U}` (`ManaPayload::Restricted`).
    /// The projection should still classify it as a mana ability so the client
    /// auto-taps it rather than surfacing a spurious menu entry.
    #[test]
    fn omen_hawker_restricted_mana_is_a_mana_ability() {
        let mut state = two_player_game();
        let hawker = state.add_card_to_battlefield(0, catalog::omen_hawker());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == hawker).unwrap();
        assert!(perm.abilities.iter().all(|a| a.is_mana),
            "Omen Hawker's restricted {{C}}{{U}} ability is a mana ability");
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

    /// Recently-added keyword-action effects surface descriptive ability
    /// labels (not the generic "Activate" catch-all) so the client tooltip is
    /// informative.
    #[test]
    fn keyword_action_effects_have_descriptive_labels() {
        use crate::effect::{Effect, PlayerRef, Value};
        assert_eq!(ability_effect_label(&Effect::Fateseal {
            who: PlayerRef::EachOpponent, amount: Value::Const(2) }), "Fateseal");
        assert_eq!(ability_effect_label(&Effect::Discover { n: Value::Const(3), filter: None }), "Discover");
    }

    /// A variable `-X` loyalty ability surfaces with `x_cost: true` so the
    /// client renders it as "-X" rather than the (zero) `loyalty_cost`.
    #[test]
    fn variable_x_loyalty_ability_flagged_in_view() {
        let mut state = two_player_game();
        let kasmina = state.add_card_to_battlefield(0, catalog::kasmina_enigma_sage());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == kasmina).unwrap();
        // +2 Scry, -X Fractal, -8 tutor.
        assert!(!perm.loyalty_abilities[0].x_cost, "+2 is a fixed-cost ability");
        assert!(perm.loyalty_abilities[1].x_cost, "the Fractal ability is -X");
    }

    /// WAR planeswalkers surface their loyalty abilities with the right
    /// targeting flags: The Wanderer's −2 exile and Ob Nixilis's −2 destroy
    /// arm the cursor; Tibalt's token-making −2 does not.
    #[test]
    fn war_planeswalker_loyalty_targeting_flags() {
        let mut state = two_player_game();
        let wanderer = state.add_card_to_battlefield(0, catalog::the_wanderer());
        let tibalt = state.add_card_to_battlefield(0, catalog::tibalt_rakish_instigator());
        let view = project(&state, 0);
        let w = view.battlefield.iter().find(|p| p.id == wanderer).unwrap();
        assert_eq!(w.loyalty_abilities[0].effect_label, "Exile permanent");
        assert!(w.loyalty_abilities[0].needs_target, "−2 exile targets a creature");
        let t = view.battlefield.iter().find(|p| p.id == tibalt).unwrap();
        assert_eq!(t.loyalty_abilities[0].effect_label, "Create token");
        assert!(!t.loyalty_abilities[0].needs_target, "the Devil-making −2 is untargeted");
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
        // Slaughter Singer's "whenever another creature you control
        // attacks" trigger is scoped `AnotherOfYours` on
        // `EventKind::Attacks`. The view should surface this as
        // "Another attacks: …" so the client tooltip renders nicely.
        // (The old fixture, Sparring Regimen, lost its synthesized
        // Attacks trigger when rewritten to the real "whenever you
        // attack" oracle.)
        let mut state = two_player_game();
        let id = state.add_card_to_battlefield(0, catalog::slaughter_singer());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == id).unwrap();
        assert!(
            perm.triggered_ability_labels.iter().any(|s| s.starts_with("Another attacks")),
            "expected 'Another attacks' label for Slaughter Singer's Attacks/AnotherOfYours trigger; got {:?}",
            perm.triggered_ability_labels,
        );
    }

    #[test]
    fn spellcast_trigger_label_distinguishes_creature_cast_from_magecraft() {
        // Halcyon Glaze's "whenever you cast a creature spell" trigger reads
        // "Creature cast", not "Magecraft" (the instant/sorcery gate).
        let mut state = two_player_game();
        let glaze = state.add_card_to_battlefield(0, catalog::halcyon_glaze());
        let view = project(&state, 0);
        let perm = view.battlefield.iter().find(|p| p.id == glaze).unwrap();
        assert!(
            perm.triggered_ability_labels.iter().any(|s| s.starts_with("Creature cast")),
            "expected 'Creature cast'; got {:?}",
            perm.triggered_ability_labels,
        );
        // A real magecraft card still reads "Magecraft".
        let mut s2 = two_player_game();
        let mage = s2.add_card_to_battlefield(0, catalog::leonin_lightscribe());
        let v2 = project(&s2, 0);
        let mp = v2.battlefield.iter().find(|p| p.id == mage).unwrap();
        assert!(
            mp.triggered_ability_labels.iter().any(|s| s.starts_with("Magecraft")),
            "expected 'Magecraft'; got {:?}",
            mp.triggered_ability_labels,
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
            monarch_guard: None,
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
            miracle: false,
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
            statics: vec![],
        });
        let view = project(&state, 0);
        assert_eq!(view.players[0].emblems, vec!["Professor Dellian Fel".to_string()]);
    }

    #[test]
    fn project_surfaces_static_emblem_ability_text() {
        use crate::card::StaticAbility;
        use crate::effect::{Selector, StaticEffect};
        let mut state = two_player_game();
        state.players[0].emblems.push(crate::player::Emblem {
            name: "Vivien Reid".into(),
            triggered: vec![],
            statics: vec![StaticAbility {
                description: "Creatures you control get +2/+2.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::This,
                    power: 2,
                    toughness: 2,
                },
            }],
        });
        let view = project(&state, 0);
        assert_eq!(
            view.players[0].emblems,
            vec!["Vivien Reid — Creatures you control get +2/+2.".to_string()]
        );
    }

    #[test]
    fn spell_cast_lock_surfaces_after_casting_the_locked_category() {
        // Deafening Silence in play, no spell cast yet → not reached.
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::deafening_silence());
        let before = project(&g, 0).players[0].spell_cast_lock.clone();
        assert!(!before.noncreature_reached, "lock not reached before any cast");
        // Cast a noncreature spell; the lock is now reached for player 0.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.perform_action(crate::game::GameAction::CastSpell {
            card_id: bolt, target: Some(crate::game::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast bolt");
        crate::game::drain_stack(&mut g);
        let after = project(&g, 0).players[0].spell_cast_lock.clone();
        assert!(after.noncreature_reached, "noncreature lock reached after a noncreature cast");
        assert!(!after.any_reached, "no Rule of Law in play → the any-spell lock stays clear");
    }

    #[test]
    fn single_combat_creature_pw_lock_surfaces_in_view() {
        let mut g = two_player_game();
        assert!(!project(&g, 0).players[0].spell_cast_lock.creature_pw_locked);
        let sc = g.add_card_to_hand(0, catalog::single_combat());
        g.step = crate::game::TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add(crate::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(crate::game::GameAction::CastSpell {
            card_id: sc, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Single Combat");
        crate::game::drain_stack(&mut g);
        assert!(project(&g, 0).players[0].spell_cast_lock.creature_pw_locked, "lock surfaces for every seat");
        assert!(project(&g, 1).players[1].spell_cast_lock.creature_pw_locked);
    }

    #[test]
    fn void_winnower_surfaces_even_mv_cast_lock() {
        // No Void Winnower → not locked; the opponent's Void Winnower locks
        // player 0's even-mv casts (CR 601.3e).
        let mut g = two_player_game();
        assert!(!project(&g, 0).players[0].even_mv_cast_locked, "unlocked without the source");
        g.add_card_to_battlefield(1, catalog::void_winnower());
        assert!(project(&g, 0).players[0].even_mv_cast_locked, "opponent's Void Winnower locks");
        // The Void Winnower controller is unaffected by their own static.
        assert!(!project(&g, 1).players[1].even_mv_cast_locked, "the controller isn't locked");
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

        // Warp (Haliya) is a plain alt cost surfaced with its {W} label.
        let haliya = crate::card::CardInstance::new(
            crate::card::CardId(3), catalog::haliya_guided_by_light(), 0);
        let k3 = known_card(&haliya);
        assert!(k3.has_alternative_cost && !k3.alt_cost_needs_pitch);
        assert_eq!(k3.alt_cost_label, "{W}", "warp cost label rendered");
    }

    #[test]
    fn alt_cost_availability_tracks_the_condition_gate() {
        // Prowl is unavailable before tribal combat damage, available after.
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::latchkey_faerie());
        let hand_view = |g: &crate::game::GameState| {
            let v = project(g, 0);
            match v.players[0].hand.iter().find(|h| matches!(h,
                crate::net::HandCardView::Known(k) if k.id == id)).unwrap()
            {
                crate::net::HandCardView::Known(k) => k.clone(),
                _ => unreachable!(),
            }
        };
        let k = hand_view(&g);
        assert!(k.has_alternative_cost && !k.alt_cost_available, "prowl gated off");
        g.players[0].prowl_types_this_turn.push(crate::card::CreatureType::Rogue);
        let k = hand_view(&g);
        assert!(k.alt_cost_available, "prowl available after a Rogue connected");
    }

    #[test]
    fn exile_view_surfaces_cipher_encoding() {
        // CR 702.46 — an exiled card encoded on a creature surfaces its carrier.
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mut card =
            crate::card::CardInstance::new(crate::card::CardId(999), catalog::shadow_slice(), 0);
        card.encoded_on = Some(bear);
        g.exile.push(card);
        let view = project(&g, 0);
        assert!(
            view.exile.iter().any(|e| e.encoded_on == Some(bear)),
            "exile view surfaces the cipher carrier"
        );
    }

    /// CR 702.187 — the graveyard view only flags a Mayhem cost once the owner
    /// has discarded that card this turn.
    #[test]
    fn graveyard_view_surfaces_mayhem_only_after_discard() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_graveyard(0, catalog::electros_bolt());
        // In the graveyard but not discarded this turn → no affordance.
        let v0 = project(&g, 0);
        let e0 = v0.players[0].graveyard.iter().find(|c| c.id == bolt).expect("bolt in graveyard view");
        assert!(e0.mayhem_cost.is_none(), "no Mayhem affordance before a discard");
        // Mark it discarded this turn → the affordance surfaces.
        g.players[0].discarded_this_turn.insert(bolt);
        let v1 = project(&g, 0);
        let e1 = v1.players[0].graveyard.iter().find(|c| c.id == bolt).expect("bolt in graveyard view");
        assert_eq!(e1.mayhem_cost.as_ref().map(|c| c.cmc()), Some(2), "the mayhem cost surfaced");
    }

    /// CR 208.3 — a noncreature `*`-power Vehicle surfaces its live power and is
    /// flagged `pt_modified` once the board pushes it off its printed base.
    #[test]
    fn vehicle_dynamic_power_surfaces_in_view() {
        let mut g = two_player_game();
        let wagon = g.add_card_to_battlefield(0, catalog::lumbering_worldwagon());
        for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
        let view = project(&g, 0);
        let pv = view.battlefield.iter().find(|p| p.id == wagon).expect("wagon in view");
        assert_eq!(pv.power, 3, "power = lands controlled");
        assert!(pv.pt_modified, "noncreature Vehicle flagged as P/T-modified");
    }

    /// CR 702.122e/702.171 — Deathless Pilot's crew-power rider surfaces in the
    /// view so the client can badge "crews as +2".
    #[test]
    fn crew_power_bonus_surfaces_in_view() {
        let mut g = two_player_game();
        let pilot = g.add_card_to_battlefield(0, catalog::deathless_pilot());
        let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let view = project(&g, 0);
        let pv = view.battlefield.iter().find(|p| p.id == pilot).expect("pilot in view");
        assert_eq!(pv.crew_power_bonus, 2, "rider surfaces");
        let bv = view.battlefield.iter().find(|p| p.id == plain).expect("bears in view");
        assert_eq!(bv.crew_power_bonus, 0, "no rider on a plain creature");
    }

    /// PlayerView surfaces the elimination cause so the HUD can annotate an
    /// eliminated portrait with why the player lost (CR 104.3).
    #[test]
    fn loss_reason_surfaces_in_player_view() {
        let mut g = two_player_game();
        assert_eq!(project(&g, 0).players[1].loss_reason, None, "still in the game");
        g.concede(1);
        assert_eq!(
            project(&g, 0).players[1].loss_reason.as_deref(),
            Some("conceded"),
            "concession is surfaced as its own cause",
        );
    }

    /// PlayerView surfaces the starting-life total so the HUD can render
    /// "N above starting" gates (CR 103.4 — Speaker of the Heavens).
    #[test]
    fn starting_life_surfaces_in_player_view() {
        let mut g = two_player_game();
        g.players[0].starting_life = 40;
        g.players[0].life = 47;
        let pv = &project(&g, 0).players[0];
        assert_eq!(pv.starting_life, 40);
        assert_eq!(pv.life - pv.starting_life, 7, "HUD can show +7 above start");
    }

    /// A non-mana alternative cost surfaces a descriptive label (Escape
    /// Detection's freerunning is "return a creature", not blank).
    #[test]
    fn alt_cost_label_describes_non_mana_riders() {
        let card = crate::card::CardInstance::new(
            crate::card::CardId(1), catalog::escape_detection(), 0);
        let k = known_card(&card);
        assert!(k.has_alternative_cost);
        assert!(k.alt_cost_label.contains("Return"),
            "alt-cost label describes the return rider, got {:?}", k.alt_cost_label);
    }

    /// The view labels a "deals combat damage to a planeswalker" trigger
    /// (Vraska, Swarm's Eminence) rather than leaving it blank.
    #[test]
    fn trigger_label_covers_combat_damage_to_planeswalker() {
        let mut g = two_player_game();
        let vraska = g.add_card_to_battlefield(0, catalog::vraska_swarms_eminence());
        let view = project(&g, 0);
        let perm = view.battlefield.iter().find(|p| p.id == vraska).unwrap();
        assert!(
            perm.triggered_ability_labels.iter().any(|s| s.contains("combat dmg to PW")),
            "expected a combat-damage-to-planeswalker label; got {:?}",
            perm.triggered_ability_labels,
        );
    }

    /// A permanent animated into a creature (Awakening of Vitu-Ghazi's land)
    /// surfaces `pt_modified` so the client draws its P/T box — even though its
    /// printed type isn't a creature.
    #[test]
    fn animated_noncreature_shows_pt_box() {
        use crate::card::{CounterType, CreatureType};
        use crate::effect::{Duration, Selector, Value};
        use crate::game::effects::EffectContext;
        use crate::game::types::Target;
        let mut g = two_player_game();
        let land = g.add_card_to_battlefield(0, catalog::forest());
        g.battlefield_find_mut(land).unwrap().add_counters(CounterType::PlusOnePlusOne, 9);
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(land)), 0, 0);
        g.resolve_effect(&Effect::BecomeCreature {
            what: Selector::Target(0),
            power: Value::ZERO,
            toughness: Value::ZERO,
            creature_types: vec![CreatureType::Elemental],
            keywords: vec![],
            duration: Duration::Permanent,
        }, &ctx).unwrap();
        let view = project(&g, 0);
        let pv = view.battlefield.iter().find(|p| p.id == land).unwrap();
        assert!(pv.pt_modified, "animated 9/9 land flags its P/T box");
    }
}
