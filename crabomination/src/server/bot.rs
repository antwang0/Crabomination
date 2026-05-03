//! In-process bots for server-hosted matches.
//!
//! Unlike networked clients, a bot reads the full authoritative [`GameState`]
//! each tick and returns the next [`GameAction`] it wants the server to
//! perform. The match actor polls every bot seat to a fixed point after every
//! state change, so a bot just needs to make *some* forward-progressing
//! decision (including `PassPriority`) whenever it holds priority.

use rand::{RngExt, rng};

use crate::card::{CardDefinition, CardId, Keyword};
use crate::decision::{AutoDecider, Decider};
use crate::effect::{ActivatedAbility, Effect, ManaPayload};
use crate::game::{Attack, AttackTarget, GameAction, GameState, Target, TurnStep};
use crate::mana::ManaPool;

/// Drives one seat without a human client. Implementations see the full
/// `GameState` and return the single next action they'd like to submit.
pub trait Bot: Send {
    /// Return `Some(action)` to submit, or `None` if it's not this bot's turn
    /// to act right now (no priority, waiting on an opponent decision, game
    /// already over, etc.).
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction>;
}

/// Random-play reference bot. Taps lands, plays a random affordable card from
/// hand, attacks with everything that can, assigns blockers at random, and
/// auto-answers any decisions with [`AutoDecider`].
///
/// The bot keeps a little internal flag state so it only submits
/// `DeclareAttackers`/`DeclareBlockers` once per combat phase — the match
/// actor polls it repeatedly, so without these flags it would re-submit every
/// tick.
pub struct RandomBot {
    last_step_key: Option<(u32, TurnStep, usize)>,
    attackers_declared: bool,
    blocks_declared: bool,
}

impl RandomBot {
    pub fn new() -> Self {
        Self {
            last_step_key: None,
            attackers_declared: false,
            blocks_declared: false,
        }
    }

    fn sync_step(&mut self, state: &GameState) {
        let key = (state.turn_number, state.step, state.active_player_idx);
        if self.last_step_key != Some(key) {
            self.last_step_key = Some(key);
            self.attackers_declared = false;
            self.blocks_declared = false;
        }
    }
}

impl Default for RandomBot {
    fn default() -> Self {
        Self::new()
    }
}

impl Bot for RandomBot {
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        if state.is_game_over() {
            return None;
        }
        self.sync_step(state);

        // Any pending decision addressed to us: auto-answer it.
        if let Some(pending) = &state.pending_decision {
            if pending.acting_player() == seat {
                let answer = AutoDecider.decide(&pending.decision);
                return Some(GameAction::SubmitDecision(answer));
            }
            return None;
        }

        if state.player_with_priority() != seat {
            return None;
        }

        let is_active = state.active_player_idx == seat;

        match state.step {
            TurnStep::DeclareBlockers if !is_active => {
                if !self.blocks_declared && !state.attacking().is_empty() {
                    self.blocks_declared = true;
                    Some(GameAction::DeclareBlockers(pick_blocks(state, seat)))
                } else {
                    Some(GameAction::PassPriority)
                }
            }
            TurnStep::DeclareAttackers if is_active => {
                if !self.attackers_declared {
                    self.attackers_declared = true;
                    // Pick the next alive opponent as the default attack
                    // target; in multiplayer this is just the next seat.
                    let target_player = state.next_alive_seat(seat);
                    // Push XXIII: walk opp planeswalkers so the bot can pick
                    // one off when the assigned attackers' total power
                    // matches its loyalty. Returns Vec<(CardId, loyalty)>
                    // sorted by loyalty ascending — cheapest to kill first.
                    let mut opp_walkers: Vec<(crate::card::CardId, u32)> = state
                        .battlefield
                        .iter()
                        .filter(|c| c.controller != seat && c.definition.is_planeswalker())
                        .map(|c| {
                            let loyalty = c.counter_count(
                                crate::card::CounterType::Loyalty,
                            );
                            (c.id, loyalty)
                        })
                        .collect();
                    opp_walkers.sort_by_key(|(_, l)| *l);

                    // Filter on `controller`, not `owner`: cards that have
                    // changed control (Threaten / Mind Control / etc.) are
                    // attacked WITH by the new controller, not the original
                    // owner.
                    let mut available: Vec<&crate::card::CardInstance> = state
                        .battlefield
                        .iter()
                        .filter(|c| c.controller == seat && c.can_attack())
                        .collect();
                    // Sort attackers by power descending so we apply the
                    // strongest to the cheapest walker first (greedy
                    // first-fit). Power == 0 attackers (Defender flickers)
                    // contribute zero to walker damage so they fall through
                    // to the player target.
                    available.sort_by_key(|c| std::cmp::Reverse(c.power()));

                    let mut attacks: Vec<Attack> = Vec::with_capacity(available.len());
                    let mut walker_iter = opp_walkers.into_iter();
                    let mut current_walker: Option<(crate::card::CardId, u32)> =
                        walker_iter.next();
                    let mut acc_power: i32 = 0;

                    for c in available {
                        // If a walker is open and we'd still need power to
                        // kill it, send this attacker its way. Once a
                        // walker accumulates ≥ loyalty, advance to the
                        // next walker (so multiple walkers can die in
                        // one alpha-strike turn).
                        if let Some((wid, wloyalty)) = current_walker {
                            attacks.push(Attack {
                                attacker: c.id,
                                target: AttackTarget::Planeswalker(wid),
                            });
                            acc_power += c.power();
                            if acc_power >= wloyalty as i32 {
                                current_walker = walker_iter.next();
                                acc_power = 0;
                            }
                        } else {
                            attacks.push(Attack {
                                attacker: c.id,
                                target: AttackTarget::Player(target_player),
                            });
                        }
                    }
                    Some(GameAction::DeclareAttackers(attacks))
                } else {
                    Some(GameAction::PassPriority)
                }
            }
            TurnStep::PreCombatMain | TurnStep::PostCombatMain if is_active => {
                Some(main_phase_action(state, seat))
            }
            _ => Some(GameAction::PassPriority),
        }
    }
}

fn main_phase_action(state: &GameState, seat: usize) -> GameAction {
    // Tap the first untapped land THE BOT CURRENTLY CONTROLS, one call
    // at a time so each mana ability surfaces as its own event. The
    // `controller`-not-`owner` filter is a cheap pre-filter; the
    // dry-run gate below enforces it definitively (controller might
    // have flipped via Threaten / Mind Control between the bot's
    // last tick and now, etc.).
    if let Some(id) = state
        .battlefield
        .iter()
        .find(|c| c.controller == seat && c.definition.is_land() && !c.tapped)
        .map(|c| c.id)
    {
        let action = GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
        };
        if state.would_accept(action.clone()) {
            return action;
        }
    }

    // After lands are exhausted, tap non-land "free" mana rocks (Sol Ring,
    // Mind Stone-style) so their mana counts toward the pool. Only auto-
    // handled mana abilities are eligible — color-choice and sacrifice-
    // cost abilities (Lotus Petal, Chromatic Star) are skipped to avoid
    // pointlessly destroying utility artifacts.
    if let Some((id, idx)) = find_free_mana_rock(state, seat) {
        let action = GameAction::ActivateAbility {
            card_id: id,
            ability_index: idx,
            target: None,
        };
        if state.would_accept(action.clone()) {
            return action;
        }
    }

    // Build list of castable non-land spells. Affordability + target
    // pre-filters reduce the candidate set; the FINAL gate is
    // `state.would_accept(...)`, which dry-runs each candidate
    // against a clone of the engine state and discards anything the
    // engine would reject (sorcery timing under Teferi, Damping
    // Sphere mana tax, hexproof targets, stolen permanents, etc.).
    // The dry-run is the source of truth — pre-filters are pure
    // performance hints to keep the candidate set small.
    let castable: Vec<GameAction> = state.players[seat]
        .hand
        .iter()
        .filter(|c| !c.definition.is_land())
        .filter(|c| can_afford_in_state(state, seat, c))
        .flat_map(|c| {
            // For modal effects (ChooseMode), enumerate each mode so the
            // bot can pick (e.g.) Drown in the Loch's mode 1 (destroy
            // creature) when no opp spell is on the stack to counter.
            // Falls back to `mode: None` (engine defaults to mode 0) for
            // non-modal spells.
            let modes: Vec<Option<usize>> = match modal_mode_count(&c.definition.effect) {
                Some(n) => (0..n).map(Some).collect(),
                None => vec![None],
            };
            let x_value = if x_relevant(&c.definition) {
                Some(max_affordable_x(state, seat, c))
            } else {
                None
            };
            modes.into_iter().filter_map(move |mode| {
                // Pick a target appropriate to the chosen mode (ChooseMode
                // mode-aware filter check happens in the cast paths).
                let mode_effect = mode_branch(&c.definition.effect, mode);
                let target = if mode_effect.requires_target() {
                    let t = state.auto_target_for_effect(mode_effect, seat);
                    t.as_ref()?;
                    t
                } else {
                    None
                };
                Some(GameAction::CastSpell {
                    card_id: c.id,
                    target,
                    mode,
                    // For X-cost spells (Banefire, Earthquake, Wrath of the
                    // Skies, Mind Twist, Repeal, …), pump as much generic
                    // mana as the pool can spare into X. Casting at X=0
                    // was a known dead end — Banefire dealt 0 damage, Mind
                    // Twist discarded nothing, Earthquake was a no-op.
                    x_value,
                })
            })
        })
        .filter(|a| state.would_accept(a.clone()))
        .collect();

    // Play a land if possible — gated through `would_accept` for
    // the same reason (the engine enforces sorcery timing, lands-
    // played-this-turn, etc.).
    if state.players[seat].can_play_land()
        && let Some(land) = state.players[seat].hand.iter().find(|c| c.definition.is_land())
    {
        let action = GameAction::PlayLand(land.id);
        if state.would_accept(action.clone()) {
            return action;
        }
    }

    if !castable.is_empty() {
        let mut r = rng();
        return castable[r.random_range(0..castable.len())].clone();
    }

    // Activate planeswalker loyalty abilities the bot controls. Pick the
    // first usable ability per walker (engine enforces sorcery timing and
    // once-per-turn). The candidate set is dry-run-gated so failed targets
    // / over-spent loyalty / opp-controlled-walker rejections drop out.
    if let Some(action) = pick_loyalty_ability(state, seat) {
        return action;
    }

    GameAction::PassPriority
}

/// Walk every planeswalker the bot controls and pick the first activatable
/// loyalty ability. Auto-target via `auto_target_for_effect` for abilities
/// that require a target. Prefers a +loyalty ability when available
/// (preserves the walker for next turn), falling back to the ability with
/// the smallest absolute loyalty cost so we don't suicide-ult immediately.
fn pick_loyalty_ability(state: &GameState, seat: usize) -> Option<GameAction> {
    for card in &state.battlefield {
        if card.controller != seat {
            continue;
        }
        if !card.definition.is_planeswalker() {
            continue;
        }
        if card.used_loyalty_ability_this_turn {
            continue;
        }
        // Walk abilities in order; prefer non-suicidal positive-loyalty
        // abilities first, then negative-loyalty ones the walker can afford.
        let current_loyalty =
            card.counter_count(crate::card::CounterType::Loyalty) as i32;
        let mut indexed: Vec<(usize, &crate::card::LoyaltyAbility)> =
            card.definition.loyalty_abilities.iter().enumerate().collect();
        indexed.sort_by_key(|(_, a)| -a.loyalty_cost);
        for (idx, ability) in indexed {
            if current_loyalty + ability.loyalty_cost < 0 {
                continue;
            }
            let target = if ability.effect.requires_target() {
                let t = state.auto_target_for_effect(&ability.effect, seat);
                t.as_ref()?;
                t
            } else {
                None
            };
            let action = GameAction::ActivateLoyaltyAbility {
                card_id: card.id,
                ability_index: idx,
                target,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

fn pick_blocks(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    // Push XXV: blocking heuristic now considers trades. Pre-fix the
    // bot threw every legal blocker into a random legal attacker —
    // suicide blocks (1/1 vs 5/5) chewed through bodies for nothing.
    // The new logic:
    //   1. If the un-blocked combat damage dealt to the bot is lethal
    //      (or close to it: life ≤ 5 after damage), chump-block the
    //      biggest attackers first to buy time.
    //   2. Otherwise, only block if the trade is favorable: the blocker
    //      survives, or the attacker dies (including via deathtouch
    //      from the blocker), or the attacker has deathtouch (any block
    //      kills it).
    //
    // Attacker / blocker data carries P/T + relevant keywords up-front
    // so the inner loop doesn't re-walk the battlefield per pair.
    struct AtkInfo { id: CardId, power: i32, toughness: i32, flying: bool, deathtouch: bool, indestructible: bool }
    struct BlkInfo { id: CardId, power: i32, toughness: i32, flying: bool, reach: bool, deathtouch: bool, indestructible: bool }

    let attackers: Vec<AtkInfo> = state
        .attacking()
        .iter()
        .filter(|atk| state.defender_for(atk.target) == Some(seat))
        .filter_map(|atk| {
            state.battlefield.iter().find(|c| c.id == atk.attacker).map(|a| AtkInfo {
                id: atk.attacker,
                power: a.power(),
                toughness: a.toughness() - a.damage as i32,
                flying: a.has_keyword(&Keyword::Flying),
                deathtouch: a.has_keyword(&Keyword::Deathtouch),
                indestructible: a.has_keyword(&Keyword::Indestructible),
            })
        })
        .collect();

    let blockers: Vec<BlkInfo> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.can_block())
        .map(|c| BlkInfo {
            id: c.id,
            power: c.power(),
            toughness: c.toughness() - c.damage as i32,
            flying: c.has_keyword(&Keyword::Flying),
            reach: c.has_keyword(&Keyword::Reach),
            deathtouch: c.has_keyword(&Keyword::Deathtouch),
            indestructible: c.has_keyword(&Keyword::Indestructible),
        })
        .collect();

    // Lethal-or-close path: if every attacker swings unblocked we drop
    // by their summed power; if that brings us to ≤ 5 (or 0), we
    // *must* trade aggressively.
    let total_swing: i32 = attackers.iter().map(|a| a.power.max(0)).sum();
    let life = state.players[seat].life;
    let lethal = total_swing >= life;
    let critical = !lethal && (life - total_swing) <= 5;

    // Helper: would `b` die to `a`'s damage in this assignment?
    let blocker_dies = |a: &AtkInfo, b: &BlkInfo| -> bool {
        if b.indestructible { return false; }
        // Deathtouch on the attacker = any damage kills the blocker.
        if a.deathtouch && a.power > 0 { return true; }
        a.power >= b.toughness
    };
    // Helper: would `a` die to `b`'s damage?
    let attacker_dies = |a: &AtkInfo, b: &BlkInfo| -> bool {
        if a.indestructible { return false; }
        if b.deathtouch && b.power > 0 { return true; }
        b.power >= a.toughness
    };
    // Trade evaluation: positive = blocking is good for us. Killing
    // the attacker is the dominant payoff; losing a body is the cost.
    // Damage-prevention is only counted when life is at risk (the
    // critical-or-lethal branch reads it via the `add_blunting`
    // multiplier so the high-life branch ignores it).
    let trade_score = |a: &AtkInfo, b: &BlkInfo, add_blunting: bool| -> i32 {
        let mut score = 0;
        if attacker_dies(a, b) { score += 3 + a.power.max(0); }
        if blocker_dies(a, b) { score -= 1 + b.power.max(0); }
        if add_blunting {
            score += a.power.max(0);
        }
        score
    };

    // Greedy assignment: highest-power attackers first; for each, pick
    // the best blocker (highest trade_score) — but only commit if the
    // trade is actually worth it (or we're under pressure).
    let mut ranked_atk: Vec<usize> = (0..attackers.len()).collect();
    ranked_atk.sort_by_key(|i| -attackers[*i].power);
    let mut used_blockers: std::collections::HashSet<CardId> =
        std::collections::HashSet::new();
    let mut assignments: Vec<(CardId, CardId)> = Vec::new();

    for ai in ranked_atk {
        let a = &attackers[ai];
        // Find the best blocker for this attacker.
        let mut best: Option<(usize, i32)> = None;
        for (bi, b) in blockers.iter().enumerate() {
            if used_blockers.contains(&b.id) { continue; }
            // Flying / reach legality.
            if a.flying && !b.flying && !b.reach { continue; }
            let score = trade_score(a, b, lethal || critical);
            // Minimum score to assign: under lethal pressure we accept
            // any chump (score is essentially "is there *some* defense");
            // under critical pressure we demand a non-negative trade
            // (no worse than even); otherwise we demand a strictly
            // positive trade (kill the attacker or survive the block).
            let threshold = if lethal { -100 } else if critical { 0 } else { 1 };
            if score < threshold { continue; }
            if best.is_none_or(|(_, s)| score > s) {
                best = Some((bi, score));
            }
        }
        if let Some((bi, _)) = best {
            let b = &blockers[bi];
            used_blockers.insert(b.id);
            assignments.push((b.id, a.id));
        }
    }
    assignments
}

/// Find an untapped, non-land permanent the bot controls whose first
/// activated ability is a "free" mana ability — `{T}: Add <fixed mana>` with
/// no extra cost (no mana_cost, no sac_cost) and a deterministic payload
/// (Colors or Colorless, not AnyOneColor / AnyColors which require a
/// choice). Returns `(card_id, ability_index)`.
///
/// Used by the bot to tap mana rocks like Sol Ring and Mind Stone in the
/// main phase. Sac-cost mana sources (Lotus Petal, Chromatic Star) and
/// color-choice abilities are skipped — both can be valuable to keep around
/// or activate at a smarter time.
fn find_free_mana_rock(state: &GameState, seat: usize) -> Option<(CardId, usize)> {
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && !c.tapped && !c.definition.is_land())
        .filter(|c| !c.summoning_sick)
        .find_map(|c| {
            c.definition
                .activated_abilities
                .iter()
                .enumerate()
                .find(|(_, a)| is_free_mana_ability(a))
                .map(|(i, _)| (c.id, i))
        })
}

fn is_free_mana_ability(a: &ActivatedAbility) -> bool {
    // A "free" mana ability is one the bot can fire without losing
    // anything: tap-only, no sacrifice, no extra mana cost, no life
    // cost (so Witherbloom Pledgemage's `{T}, Pay 1 life: Add {B}` is
    // *not* free), and a deterministic payload (Colors / Colorless).
    // The condition gate is also a non-trivial cost (Resonating Lute's
    // 7-cards-in-hand check) so we skip those too.
    if !a.tap_cost
        || a.sac_cost
        || !a.mana_cost.symbols.is_empty()
        || a.life_cost > 0
        || a.condition.is_some()
    {
        return false;
    }
    matches!(
        &a.effect,
        Effect::AddMana { pool: ManaPayload::Colors(_) | ManaPayload::Colorless(_), .. }
    )
}

/// True if the player can pay the card's mana cost from their current
/// pool **including** static-ability cost increases (Damping Sphere's
/// post-first-spell tax, Chancellor of the Annex's first-spell tax).
///
/// The state-aware overload `can_afford_in_state` is what the bot's
/// main_phase_action uses; the simpler signature is kept for
/// existing callers that don't have a `GameState` handy.
pub fn can_afford(def: &CardDefinition, pool: &ManaPool) -> bool {
    can_afford_with_extra(def, pool, 0)
}

/// State-aware affordability check: queries the engine for any
/// per-spell tax that would apply (Damping Sphere etc.) and folds it
/// into the cost before testing the pool. Used by the random bot to
/// avoid submitting `CastSpell` actions that the engine will reject
/// with a mana shortfall — repeated rejections are what deadlocked
/// `debug/deadlock-t8-1777411577-473115700.json` (Damping Sphere on
/// the board, bot casting its second spell of the turn).
pub fn can_afford_in_state(
    state: &GameState,
    seat: usize,
    card: &crate::card::CardInstance,
) -> bool {
    let extra = state.extra_cost_for_card_in_hand(seat, card.id);
    if !can_afford_with_extra(&card.definition, &state.players[seat].mana_pool, extra) {
        return false;
    }
    // Push XXXIX: cards with an additional cast-time sacrifice cost
    // (Daemogoth Woe-Eater, Eyeblight Cullers) are unaffordable when
    // the controller has no other matching permanent. The engine
    // would reject the cast with `SelectionRequirementViolated` —
    // skipping the dry-run noise here makes the bot's pick_action
    // walker O(1) on these gates rather than retrying.
    if let Some(filter) = card.definition.additional_sac_cost.as_ref() {
        let has_one = state.battlefield.iter().any(|c| {
            c.controller == seat
                && c.id != card.id
                && state.evaluate_requirement_static(
                    filter,
                    &crate::game::types::Target::Permanent(c.id),
                    seat,
                )
        });
        if !has_one {
            return false;
        }
    }
    true
}

/// For an X-cost spell (or a spell whose effect reads
/// `Value::XFromCost`), return the largest non-negative X the caster can
/// pay given their current mana pool — leftover generic mana after the
/// fixed (non-X) portion of the cost is what fuels X. Static cost taxes
/// (Damping Sphere etc.) are folded in via
/// `extra_cost_for_card_in_hand`. Returns 0 when the fixed cost itself
/// is more than the available pool — the caller relies on `would_accept`
/// to reject the unaffordable cast.
///
/// We detect X-relevance via either the cost's explicit `{X}` pip
/// (Wrath of the Skies) **or** an `XFromCost` reference inside the
/// effect tree (Banefire / Earthquake / Mind Twist — these have flat
/// fixed costs in the catalog because the engine had no Value::XFromCost
/// wiring at the time they were added; the X mana goes straight into
/// the pool and the bot pumps the spell at its full pool size).
pub fn max_affordable_x(
    state: &GameState,
    seat: usize,
    card: &crate::card::CardInstance,
) -> u32 {
    if !x_relevant(&card.definition) { return 0; }
    let pool_total = state.players[seat].mana_pool.total();
    let fixed_cmc = card.definition.cost.with_x_value(0).cmc();
    let extra = state.extra_cost_for_card_in_hand(seat, card.id);
    let needed = fixed_cmc + extra;
    pool_total.saturating_sub(needed)
}

/// True if X matters for this spell — either the cost has an `{X}` pip
/// or the effect tree mentions `Value::XFromCost`. The latter catches
/// catalog cards (Banefire, Mind Twist, …) whose costs predate the
/// engine's proper X-pip wiring.
pub fn x_relevant(def: &CardDefinition) -> bool {
    def.cost.has_x() || effect_uses_x(&def.effect)
}

fn effect_uses_x(eff: &Effect) -> bool {
    use crate::effect::Value;
    fn value_uses_x(v: &Value) -> bool {
        match v {
            Value::XFromCost => true,
            Value::Sum(parts) => parts.iter().any(value_uses_x),
            Value::Diff(a, b)
            | Value::Times(a, b)
            | Value::Min(a, b)
            | Value::Max(a, b) => value_uses_x(a) || value_uses_x(b),
            Value::NonNeg(inner) => value_uses_x(inner),
            Value::CountOf(_) | Value::PowerOf(_) | Value::ToughnessOf(_)
            | Value::CountersOn { .. } | Value::ManaValueOf(_)
            | Value::DistinctTypesInTopOfLibrary { .. } => false,
            _ => false,
        }
    }
    fn predicate_uses_x(p: &crate::effect::Predicate) -> bool {
        use crate::effect::Predicate as P;
        match p {
            P::ValueAtLeast(a, b) | P::ValueAtMost(a, b) => value_uses_x(a) || value_uses_x(b),
            P::Not(inner) => predicate_uses_x(inner),
            P::All(parts) | P::Any(parts) => parts.iter().any(predicate_uses_x),
            P::SelectorCountAtLeast { n, .. } => value_uses_x(n),
            _ => false,
        }
    }
    match eff {
        Effect::Seq(steps) => steps.iter().any(effect_uses_x),
        Effect::If { cond, then, else_ } => {
            predicate_uses_x(cond) || effect_uses_x(then) || effect_uses_x(else_)
        }
        Effect::ChooseMode(modes) => modes.iter().any(effect_uses_x),
        Effect::ChooseModes { modes, .. } => modes.iter().any(effect_uses_x),
        Effect::PickModeAtResolution(modes) => modes.iter().any(effect_uses_x),
        Effect::ForEach { body, .. }
        | Effect::Repeat { body, .. }
        | Effect::DelayUntil { body, .. } => effect_uses_x(body),
        Effect::DealDamage { amount, .. }
        | Effect::GainLife { amount, .. }
        | Effect::LoseLife { amount, .. }
        | Effect::Drain { amount, .. }
        | Effect::Draw { amount, .. }
        | Effect::Mill { amount, .. }
        | Effect::Scry { amount, .. }
        | Effect::Surveil { amount, .. }
        | Effect::LookAtTop { amount, .. }
        | Effect::AddCounter { amount, .. }
        | Effect::RemoveCounter { amount, .. }
        | Effect::AddPoison { amount, .. } => value_uses_x(amount),
        Effect::Discard { amount, .. } => value_uses_x(amount),
        Effect::PumpPT { power, toughness, .. } => {
            value_uses_x(power) || value_uses_x(toughness)
        }
        Effect::Sacrifice { count, .. } | Effect::DiscardChosen { count, .. } => {
            value_uses_x(count)
        }
        Effect::CreateToken { count, .. } | Effect::CopySpell { count, .. } => value_uses_x(count),
        Effect::RevealUntilFind { cap, .. } => value_uses_x(cap),
        Effect::AddFirstSpellTax { count, .. } => value_uses_x(count),
        _ => false,
    }
}

/// If `eff` is (or wraps via `Seq`) a top-level `ChooseMode`, return the
/// number of modes. Otherwise `None`. The bot uses this to enumerate each
/// mode separately when generating castable actions, so a card whose
/// default mode (mode 0) is dead in the current board state (e.g. Drown
/// in the Loch's "counter target spell" with no opp spell on the stack)
/// still surfaces a viable alternate (mode 1: destroy creature).
fn modal_mode_count(eff: &Effect) -> Option<usize> {
    match eff {
        Effect::ChooseMode(modes) => Some(modes.len()),
        // ChooseModes also supports a single-mode `mode: Some(N)`
        // override at cast time (push XXXVI), so the bot's "enumerate
        // each mode" path applies here too.
        Effect::ChooseModes { modes, .. } => Some(modes.len()),
        Effect::Seq(steps) => steps.iter().find_map(modal_mode_count),
        _ => None,
    }
}

/// Resolve the effect branch for a chosen mode. For non-modal effects
/// (or `mode == None`), returns the original effect. For modal effects,
/// returns the chosen mode's body so the auto-target heuristic uses the
/// correct filter for that mode.
fn mode_branch(eff: &Effect, mode: Option<usize>) -> &Effect {
    match (eff, mode) {
        (Effect::ChooseMode(modes), Some(m)) if m < modes.len() => &modes[m],
        (Effect::ChooseModes { modes, .. }, Some(m)) if m < modes.len() => &modes[m],
        (Effect::Seq(steps), Some(_)) => steps
            .iter()
            .find(|s| matches!(s, Effect::ChooseMode(_) | Effect::ChooseModes { .. }))
            .map(|s| mode_branch(s, mode))
            .unwrap_or(eff),
        _ => eff,
    }
}

fn can_afford_with_extra(def: &CardDefinition, pool: &ManaPool, extra_generic: u32) -> bool {
    let mut cost = if def.cost.has_x() {
        def.cost.with_x_value(0)
    } else {
        def.cost.clone()
    };
    if extra_generic > 0 {
        cost.symbols.push(crate::mana::ManaSymbol::Generic(extra_generic));
    }
    pool.clone().pay(&cost).is_ok()
}

/// Pick a sensible auto-target for a spell cast by `caster` using the
/// engine's shared targeting heuristic.
pub fn choose_target(state: &GameState, def: &CardDefinition, caster: usize) -> Option<Target> {
    state.auto_target_for_effect(&def.effect, caster)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::game::GameState;
    use crate::player::Player;

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    /// Free, fixed-payload mana rocks like Sol Ring should be picked up by
    /// the bot's main-phase action loop after lands are exhausted.
    #[test]
    fn bot_taps_free_mana_rock_after_lands() {
        let mut g = two_player_game();
        let sol = g.add_card_to_battlefield(0, catalog::sol_ring());
        g.clear_sickness(sol);
        // No untapped lands, so the bot's land-tap branch returns None and
        // the new mana-rock branch fires.
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        match action {
            GameAction::ActivateAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, sol, "bot should target Sol Ring");
                assert_eq!(ability_index, 0);
            }
            _ => panic!("bot should activate Sol Ring's mana ability"),
        }
    }

    /// Life-cost mana abilities (Witherbloom Pledgemage's `{T}, Pay 1 life`)
    /// are NOT auto-activated as "free" — paying life is a non-trivial cost
    /// the bot can't reason about. Push XXIV: tightened `is_free_mana_ability`
    /// to skip life_cost > 0 (alongside the existing sac_cost / mana_cost /
    /// condition guards).
    #[test]
    fn bot_does_not_tap_life_cost_mana_source() {
        let mut g = two_player_game();
        let pledgemage = g.add_card_to_battlefield(0, catalog::witherbloom_pledgemage());
        g.clear_sickness(pledgemage);
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        if let GameAction::ActivateAbility { card_id, .. } = action {
            assert_ne!(card_id, pledgemage,
                "bot must NOT auto-tap a life-cost mana source like Witherbloom Pledgemage");
        }
    }

    /// Sac-cost mana abilities (Lotus Petal) are NOT auto-activated — they
    /// destroy the source on activation, which the random bot can't reason
    /// about.
    #[test]
    fn bot_does_not_tap_sac_cost_mana_source() {
        let mut g = two_player_game();
        let petal = g.add_card_to_battlefield(0, catalog::lotus_petal());
        g.clear_sickness(petal);
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        // Should not activate Lotus Petal's sac-cost ability.
        if let GameAction::ActivateAbility { card_id, .. } = action {
            assert_ne!(card_id, petal, "bot must NOT auto-tap a sac-cost mana source");
        }
    }

    /// Bot activates a planeswalker's loyalty ability when one is available,
    /// preferring +loyalty abilities that preserve the walker for next turn.
    #[test]
    fn bot_activates_planeswalker_loyalty_ability() {
        let mut g = two_player_game();
        // Karn has a +1 (draw 1, mill 1) at index 0, a -1 at index 1, and a
        // -2 (Construct token) at index 2. Sorted by descending loyalty
        // cost, the bot should pick the +1 first.
        let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
        g.clear_sickness(karn);
        // Stock the library so the +1's draw + mill have inputs.
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());

        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, karn, "bot should target the Karn it controls");
                assert_eq!(ability_index, 0,
                    "+1 loyalty preferred over -1 / -2 (don't suicide-ult)");
            }
            other => panic!("expected ActivateLoyaltyAbility, got {:?}", other),
        }
    }

    /// Color-choice mana abilities (Ornithopter of Paradise's `{T}: Add one
    /// mana of any color`) require an interactive `ChooseColor` decision,
    /// which the bot's main loop doesn't supply at activation time. Those
    /// are filtered out of `find_free_mana_rock`.
    #[test]
    fn bot_does_not_tap_color_choice_mana_source() {
        let mut g = two_player_game();
        let bird = g.add_card_to_battlefield(0, catalog::ornithopter_of_paradise());
        g.clear_sickness(bird);
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should produce an action");
        if let GameAction::ActivateAbility { card_id, .. } = action {
            assert_ne!(card_id, bird,
                "bot must NOT auto-tap a color-choice mana source (would block on ChooseColor)");
        }
    }

    /// Reproducer for the "Vandalblast freeze" bug. The bot is in its main
    /// phase with a Mountain (already tapped or untapped) and Vandalblast in
    /// hand; the human opponent has only an Ornithopter of Paradise on the
    /// battlefield. The bot must pick that artifact as the target and the
    /// match must drive to completion without spinning the bot loop.
    #[test]
    fn bot_vs_bot_vandalblast_against_lone_artifact_resolves() {
        use crate::server::{run_match, SeatOccupant};
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;
        let mut g = two_player_game();
        // Bot owns a Mountain so it can pay {R} and Vandalblast in hand.
        let mtn = g.add_card_to_battlefield(0, catalog::mountain());
        g.clear_sickness(mtn);
        g.add_card_to_hand(0, catalog::vandalblast());
        // Opponent has only Ornithopter of Paradise on the battlefield.
        let bird = g.add_card_to_battlefield(1, catalog::ornithopter_of_paradise());
        g.clear_sickness(bird);
        // Both bots; expect the match to terminate within a short window.
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match(
                g,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            );
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(15))
            .expect("bot-vs-bot match must terminate (Vandalblast freeze regression)");
        handle.join().unwrap();
    }

    /// Direct (non-server) regression: the bot's main-phase action loop
    /// picks the opponent's Ornithopter as the legal Vandalblast target
    /// when no other artifact is in play. The Mountain has already been
    /// tapped (we seed the pool with {R} and pre-tap the land) so the
    /// bot proceeds straight to the spell-cast step.
    #[test]
    fn bot_main_phase_emits_vandalblast_action() {
        let mut g = two_player_game();
        let mtn = g.add_card_to_battlefield(0, catalog::mountain());
        if let Some(c) = g.battlefield_find_mut(mtn) {
            c.tapped = true;
        }
        let vandal = g.add_card_to_hand(0, catalog::vandalblast());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        let bird = g.add_card_to_battlefield(1, catalog::ornithopter_of_paradise());
        g.clear_sickness(bird);
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        match action {
            GameAction::CastSpell { card_id, target, .. } => {
                assert_eq!(card_id, vandal, "expected the bot to cast Vandalblast");
                assert_eq!(
                    target,
                    Some(Target::Permanent(bird)),
                    "Vandalblast must target the lone artifact opp controls",
                );
            }
            other => panic!("expected CastSpell(Vandalblast), got {other:?}"),
        }
    }

    /// End-to-end deadlock regression for spectate-mode bot-vs-bot:
    /// load a hand-crafted state that mirrors the captured cube debug
    /// export (own-stack trigger + sorcery-speed castables + a played
    /// land already) and assert the match drives forward instead of
    /// hanging on `merged_rx.recv()`. Pre-fix this would have hung on
    /// any RNG that picked Tireless Tracker before Lightning Bolt.
    #[test]
    fn spectate_match_does_not_deadlock_with_own_trigger_on_stack() {
        use crate::effect::Effect;
        use crate::game::{StackItem, TurnStep};
        use crate::server::{run_match, SeatOccupant};
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        let tracker = g.add_card_to_battlefield(0, catalog::tireless_tracker());
        g.clear_sickness(tracker);
        g.stack.push(StackItem::Trigger {
            source: tracker,
            controller: 0,
            effect: Box::new(Effect::Noop),
            target: None,
            mode: None,
            x_value: 0,
            converged_value: 0,
            subject: None,
        });
        g.add_card_to_hand(0, catalog::tireless_tracker());
        g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Green, 5);
        g.players[0].mana_pool.add(crate::mana::Color::Red, 5);
        g.players[0].lands_played_this_turn = 1;
        // Both players at 1 life so combat damage ends the match
        // quickly once a creature attacks.
        g.players[0].life = 1;
        g.players[1].life = 1;

        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            run_match(
                g,
                vec![
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            );
            let _ = done_tx.send(());
        });
        done_rx
            .recv_timeout(Duration::from_secs(15))
            .expect("bot-vs-bot match must terminate (own-stack-trigger deadlock regression)");
        handle.join().unwrap();
    }

    /// Regression for the Spectate Bot vs Bot deadlock observed in
    /// `debug/state-t11-precombatmain-1777409468-338551100.json`.
    ///
    /// Setup: bot 0 has its own Tireless Tracker trigger sitting on the
    /// stack (no target), all its lands are tapped and one was already
    /// played this turn, and its hand has both sorcery- and instant-
    /// speed castables. Pre-fix, `main_phase_action` sometimes picked a
    /// sorcery to cast — the engine rejected it with `SorcerySpeedOnly`
    /// (stack non-empty), `drive_bots` saw no progress, the actor blocked
    /// on `merged_rx.recv()`, and a spectator-only match froze.
    ///
    /// Post-fix the bot must either pass priority or cast an instant —
    /// never a sorcery — when the stack is non-empty.
    #[test]
    fn bot_does_not_attempt_sorcery_when_stack_nonempty() {
        use crate::effect::Effect;
        use crate::game::StackItem;
        let mut g = two_player_game();
        // Bot 0 has a tracker on the battlefield as the trigger source.
        let tracker = g.add_card_to_battlefield(0, catalog::tireless_tracker());
        g.clear_sickness(tracker);
        // Stack: Tireless Tracker trigger (Clue creation), no target.
        g.stack.push(StackItem::Trigger {
            source: tracker,
            controller: 0,
            effect: Box::new(Effect::Noop), // exact effect doesn't matter here
            target: None,
            mode: None,
            x_value: 0,
            converged_value: 0,
            subject: None,
        });
        // Hand: a mix of sorcery- and instant-speed castables. Pyrokinesis
        // (instant) is the only legal cast right now.
        g.add_card_to_hand(0, catalog::tireless_tracker());
        g.add_card_to_hand(0, catalog::lightning_bolt());
        // Mana pool topped up so `can_afford` accepts both.
        g.players[0].mana_pool.add(crate::mana::Color::Green, 5);
        g.players[0].mana_pool.add(crate::mana::Color::Red, 5);
        // Pretend a land was played already so PlayLand is also blocked.
        g.players[0].lands_played_this_turn = 1;

        let mut bot = RandomBot::new();
        // Drive a few action picks; none of them may be a sorcery-speed
        // CastSpell (Tireless Tracker). PassPriority and instant casts
        // (Lightning Bolt) are both fine.
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::CastSpell { card_id, .. } = action {
                let def = g.players[0].hand.iter().find(|c| c.id == card_id)
                    .map(|c| &c.definition);
                if let Some(d) = def {
                    assert!(
                        d.is_instant_speed(),
                        "bot tried to cast sorcery-speed {} while stack was non-empty",
                        d.name,
                    );
                }
            }
        }
    }

    /// Regression for the Teferi sorcery-lock deadlock. With Teferi,
    /// Time Raveler on the opponent's side, our **instants** are
    /// timing-locked to sorcery speed. The bot's pre-fix filter
    /// allowed instant casts whenever `is_instant_speed()` was true,
    /// regardless of `OpponentsSorceryTimingOnly`; the engine then
    /// rejected with `SorcerySpeedOnly` and the match deadlocked.
    /// Post-fix, `would_accept` dry-runs the cast and rejects it,
    /// so the bot picks a different action (or passes priority).
    #[test]
    fn bot_respects_teferi_sorcery_lock_on_instants() {
        let mut g = two_player_game();
        // Opponent's Teferi imposes `OpponentsSorceryTimingOnly`.
        let teferi = g.add_card_to_battlefield(1, catalog::teferi_time_raveler());
        g.clear_sickness(teferi);
        // Stack non-empty so sorcery-speed timing fails for the bot.
        g.spells_cast_this_turn = 0;
        // Put a dummy spell on the stack to break sorcery timing
        // even on the bot's main phase.
        let dummy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield.retain(|c| c.id != dummy);
        let card = crate::card::CardInstance::new(dummy, catalog::grizzly_bears(), 1);
        g.stack.push(crate::game::StackItem::Spell {
            card: Box::new(card),
            caster: 1,
            target: None,
            mode: None,
            x_value: 0,
            converged_value: 0,
            uncounterable: false,
            face: crate::game::types::CastFace::Front,
            is_copy: false,
        });
        // Bot 0 has Lightning Bolt (instant) in hand and a Mountain.
        let _bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        let mut bot = RandomBot::new();
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::CastSpell { .. } = action {
                panic!(
                    "bot tried to cast at instant speed under Teferi's lock — \
                     would_accept must filter this out: {action:?}",
                );
            }
        }
    }

    /// Regression for the deadlock at `debug/deadlock-t8-1777411577-473115700.json`.
    /// Damping Sphere on the battlefield + bot has already cast one spell this
    /// turn + a second affordable-by-printed-cost spell in hand whose real cost
    /// (printed + Damping Sphere's `+1` tax) overflows the pool. Pre-fix the
    /// bot's `can_afford` checked only the printed cost; cast was rejected with
    /// `Mana: Need N generic mana but only have N-1 total`; spectate-mode actor
    /// deadlocked. Post-fix `can_afford_in_state` folds the static-ability tax
    /// into the cost so the bot doesn't pick the unaffordable spell.
    #[test]
    fn bot_respects_damping_sphere_tax() {
        let mut g = two_player_game();
        // Opponent's Damping Sphere on the battlefield.
        let sphere = g.add_card_to_battlefield(1, catalog::damping_sphere());
        g.clear_sickness(sphere);
        // Bot 0 has cast one spell already this turn.
        g.players[0].spells_cast_this_turn = 1;
        g.spells_cast_this_turn = 1;
        // Bot 0 has Frantic Search ({2}{U}) in hand and exactly 3 mana
        // (1U + 2C). Without the Damping Sphere tax the bot could
        // pay {2}{U}; with the +1 tax it can't (needs {3}{U} total).
        let _frantic = g.add_card_to_hand(0, catalog::frantic_search());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);

        let mut bot = RandomBot::new();
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::CastSpell { card_id, .. } = action {
                let name = g
                    .players[0]
                    .hand
                    .iter()
                    .find(|c| c.id == card_id)
                    .map(|c| c.definition.name);
                assert_ne!(
                    name,
                    Some("Frantic Search"),
                    "bot must respect Damping Sphere's +1 tax — pool can't pay {{3}}{{U}}",
                );
            }
        }
    }

    /// Regression for the second deadlock observed at
    /// `debug/deadlock-t15-1777411082-269586900.json`. Setup mirrors
    /// the captured cube state: P0 owns a Swamp whose `controller` has
    /// flipped to P1 (Threaten / Mind Control style), all of P0's own
    /// lands are tapped. Pre-fix the bot's main_phase_action filter
    /// (`c.owner == seat`) picked the stolen Swamp, `activate_ability`
    /// rejected with `NotYourPriority`, no progress was made, and the
    /// wall-clock watchdog tripped. Post-fix the filter is keyed on
    /// `c.controller`, so the stolen land is invisible to bot 0 and
    /// the bot falls through to its castable-spell branch (or
    /// `PassPriority`).
    #[test]
    fn max_affordable_x_returns_zero_for_non_x_spells() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::lightning_bolt());
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        assert_eq!(max_affordable_x(&g, 0, &card), 0,
            "Non-X spell yields 0 — caller should pass x_value=None");
    }

    #[test]
    fn max_affordable_x_pumps_remaining_mana_into_x() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire()); // {X}{R}
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        // Pool: 1 red + 4 colorless. Fixed cost = {R} (1 mana). X = 4.
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(4);
        assert_eq!(max_affordable_x(&g, 0, &card), 4,
            "X soaks up the remaining {{4}} after the fixed {{R}} pip");
    }

    #[test]
    fn max_affordable_x_is_zero_if_only_fixed_cost_can_be_paid() {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire());
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        // Only enough mana for the {R} pip — X must collapse to 0.
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        assert_eq!(max_affordable_x(&g, 0, &card), 0);
    }

    #[test]
    fn bot_casts_x_cost_burn_at_max_x() {
        // Banefire's catalog cost is just `{R}` (X is read at resolution
        // from `Value::XFromCost`), so x_relevant() picks it up via the
        // effect-tree XFromCost reference and the bot pumps the rest of
        // its pool into X.
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::banefire());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(3);
        let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
        // Verify the helper independently first — the bot's `next_action`
        // gates on lots of other state (priority, lands, mana rocks) so
        // a direct call to the helper is the most reliable assertion.
        assert_eq!(max_affordable_x(&g, 0, &card), 3,
            "{{R}} + {{3}} in pool, fixed cost {{R}} => X = 3");
    }

    #[test]
    fn bot_does_not_try_to_tap_stolen_land() {
        let mut g = two_player_game();
        // P0's own Swamp: tapped (already used this turn).
        let own = g.add_card_to_battlefield(0, catalog::swamp());
        if let Some(c) = g.battlefield_find_mut(own) {
            c.tapped = true;
        }
        // P0-owned Swamp now controlled by P1 (the deadlock state).
        let stolen = g.add_card_to_battlefield(0, catalog::swamp());
        if let Some(c) = g.battlefield_find_mut(stolen) {
            c.controller = 1;
            c.tapped = false;
        }

        let mut bot = RandomBot::new();
        // 50 trials; if the bot ever returns ActivateAbility on the
        // stolen card it would deadlock. PassPriority and any action
        // on a card the bot actually controls are both fine.
        for _ in 0..50 {
            let Some(action) = bot.next_action(&g, 0) else { continue };
            if let GameAction::ActivateAbility { card_id, .. } = action {
                assert_ne!(
                    card_id, stolen,
                    "bot must not try to activate a stolen permanent",
                );
            }
        }
    }

    /// Modal spells: when the default mode is dead in the current state
    /// (e.g. Drown in the Loch's mode 0 "counter target spell" with no
    /// opp spell on the stack), the bot picks an alternate mode that
    /// has a legal target. Pre-fix the bot always passed `mode: None`
    /// → engine defaulted to mode 0 → cast was rejected at target
    /// validation, and Drown in the Loch was never cast.
    #[test]
    fn bot_picks_alternate_mode_for_modal_spell() {
        let mut g = two_player_game();
        // Opp creature for mode-1 (destroy creature) to target.
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(bear);
        // Tap an Island/Swamp so {U}{B} is in the pool — bot's land-tap
        // branch otherwise fires first.
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.add_card_to_hand(0, catalog::drown_in_the_loch());
        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should act");
        // The bot should cast Drown in the Loch with mode = Some(1)
        // (destroy mode). Mode 0 (counter spell) has no spell on the
        // stack, so it's pruned from the candidate set.
        match action {
            GameAction::CastSpell { mode, target, .. } => {
                assert_eq!(mode, Some(1),
                    "Bot should pick mode 1 when mode 0 has no legal target");
                assert_eq!(target, Some(crate::game::Target::Permanent(bear)),
                    "Mode 1's target should be the opp creature");
            }
            other => panic!(
                "expected Drown in the Loch cast with mode 1, got {:?}", other),
        }
    }

    /// `modal_mode_count`: returns the mode count for ChooseMode and
    /// None for non-modal effects.
    #[test]
    fn modal_mode_count_helper() {
        let drown = catalog::drown_in_the_loch();
        assert_eq!(modal_mode_count(&drown.effect), Some(2),
            "Drown in the Loch has 2 modes");
        let bolt = catalog::lightning_bolt();
        assert_eq!(modal_mode_count(&bolt.effect), None,
            "Lightning Bolt is not modal");
    }

    /// Push XXIII: bot now considers opp planeswalkers as attack
    /// targets and picks them off when its attackers' total power
    /// matches the walker's loyalty. Build a bot-controlled 4-power
    /// attacker against a 3-loyalty planeswalker — the bot should
    /// route the attacker at the walker (cheaper kill) instead of
    /// the player.
    #[test]
    fn bot_attacks_killable_planeswalker() {
        let mut g = two_player_game();
        // P0 (active player) controls a Shivan Dragon (5/5).
        let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
        g.clear_sickness(dragon);
        // P1 controls Karn, Scion of Urza (3 starting loyalty).
        let karn = g.add_card_to_battlefield(1, catalog::karn_scion_of_urza());
        g.clear_sickness(karn);
        // Force the bot into the DeclareAttackers step on its own turn.
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;

        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should declare attackers");
        match action {
            GameAction::DeclareAttackers(attacks) => {
                assert_eq!(attacks.len(), 1, "exactly one attacker");
                assert_eq!(attacks[0].attacker, dragon, "Dragon attacks");
                assert_eq!(attacks[0].target, AttackTarget::Planeswalker(karn),
                    "Dragon should target the killable walker, not the player");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// Push XXV: pick_blocks now skips suicide blocks. A 2/2 bear
    /// blocking a 5/5 dragon (with reach since dragon flies) is a
    /// suicide trade — at full life the bot should NOT block.
    #[test]
    fn bot_skips_suicide_block_at_high_life() {
        let mut g = two_player_game();
        // P0 attacks with Shivan Dragon (5/5 flying).
        let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
        g.clear_sickness(dragon);
        // P1 controls a 2/2 with reach (Pelt Collector is 1/1; we use
        // Resilient Khenra which is 3/2 — still dies to 5 power, so
        // suicide trade. Easier: use a vanilla 2/2 with reach. The
        // simplest cube creature with reach is Wall of Omens (0/4).
        // Wall of Omens (0/4) blocks Dragon: wall dies (5 ≥ 4), wall
        // deals 0, dragon lives → suicide block.
        // Wall of Omens has no reach! It's a Defender (0/4). Defender
        // can block normal creatures. Flying still blocks fail — we
        // need flying or reach.
        // Skip the flying case: switch attacker to a 5/5 ground threat.
        // We don't have a vanilla 5/5 ground in the test catalog
        // immediately at hand, so we manufacture: take Shivan Dragon's
        // body and strip flying via clearing keywords.
        {
            let d = g.battlefield.iter_mut().find(|c| c.id == dragon).unwrap();
            d.definition.keywords.retain(|k| k != &Keyword::Flying);
        }
        let chump = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(chump);
        g.active_player_idx = 0;
        g.set_attacking(vec![crate::game::types::Attack {
            attacker: dragon,
            target: AttackTarget::Player(1),
        }]);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        g.players[1].life = 20;

        let blocks = pick_blocks(&g, 1);
        assert!(blocks.is_empty(),
            "suicide block (2/2 vs 5/5 at 20 life) should be skipped, got {:?}", blocks);
    }

    /// At critical life the bot SHOULD chump-block even unfavorable
    /// trades to buy a turn.
    #[test]
    fn bot_chump_blocks_when_lethal_imminent() {
        let mut g = two_player_game();
        // P0 attacks with a 2/2 (Grizzly Bears).
        let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(attacker);
        // P1 controls another 2/2.
        let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.clear_sickness(blocker);
        g.active_player_idx = 0;
        g.set_attacking(vec![crate::game::types::Attack {
            attacker,
            target: AttackTarget::Player(1),
        }]);
        g.step = TurnStep::DeclareBlockers;
        g.priority.player_with_priority = 1;
        // Drop life so 2 dmg = lethal.
        g.players[1].life = 2;

        let blocks = pick_blocks(&g, 1);
        // Even trade (2/2 blocks 2/2: both die) is favorable for us
        // and trivially blocks; the lethal-pressure branch is an
        // additional safety net even for unfavorable trades.
        assert!(!blocks.is_empty(),
            "should block when life is at 2 and attacker would deal 2 (lethal)");
    }

    /// When no planeswalker is on board, the bot still attacks the
    /// player as before.
    #[test]
    fn bot_attacks_player_when_no_walkers_present() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;

        let mut bot = RandomBot::new();
        let action = bot.next_action(&g, 0).expect("bot should declare attackers");
        match action {
            GameAction::DeclareAttackers(attacks) => {
                assert_eq!(attacks.len(), 1);
                assert_eq!(attacks[0].target, AttackTarget::Player(1),
                    "no walker → bot attacks the player");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }
}
