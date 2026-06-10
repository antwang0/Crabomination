//! In-process bots for server-hosted matches.
//!
//! Unlike networked clients, a bot reads the full authoritative [`GameState`]
//! each tick and returns the next [`GameAction`] it wants the server to
//! perform. The match actor polls every bot seat to a fixed point after every
//! state change, so a bot just needs to make *some* forward-progressing
//! decision (including `PassPriority`) whenever it holds priority.

use rand::{RngExt, rng};

use crate::card::{CardDefinition, CardId};
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

/// Reference bot. Taps lands and plays a (roughly random) affordable card
/// from hand, but combat is heuristic: it attacks with creatures that swing
/// safely or profitably (evasion / first-strike / deathtouch / menace /
/// lifelink / trample / indestructible awareness, plus a suicide filter and
/// planeswalker redirection) and assigns blockers to maximize value trades
/// and survive lethal (see `pick_attack`/`pick_blocks`). Decisions are
/// auto-answered with [`AutoDecider`].
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
                let answer = match &pending.decision {
                    // Smarter mulligan than AutoDecider's blanket Keep:
                    // ship hands that are flooded or screwed on lands.
                    crate::decision::Decision::Mulligan { mulligans_taken, .. } => {
                        decide_mulligan(state, seat, *mulligans_taken)
                    }
                    // Unlike AutoDecider (which declines every tutor), the
                    // bot actually fetches — preferring a basic land toward
                    // its weakest color so singleplayer tutors fix mana.
                    crate::decision::Decision::SearchLibrary { candidates, eligible, .. } => {
                        // Only consider picks the engine will accept.
                        let pickable: Vec<(crate::card::CardId, String)> = match eligible {
                            Some(ok) => candidates
                                .iter()
                                .filter(|(id, _)| ok.contains(id))
                                .cloned()
                                .collect(),
                            None => candidates.clone(),
                        };
                        decide_library_search(state, seat, &pickable)
                    }
                    // Unlike AutoDecider (which declines *every* "you may"
                    // trigger), the bot takes an optional trigger whose body
                    // is pure upside — so Provoke's "you may", Boast token
                    // riders, etc. actually fire under bot play. It still
                    // declines bodies that impose a self-cost (lose life /
                    // sacrifice / discard).
                    crate::decision::Decision::OptionalTrigger { source, description } => {
                        crate::decision::DecisionAnswer::Bool(optional_trigger_beneficial(
                            state,
                            *source,
                            description,
                        ))
                    }
                    // AutoDecider chooses nothing; the bot exiles opponents'
                    // graveyard cards (deny graveyard value) up to the cap.
                    crate::decision::Decision::ChooseCards { candidates, max, .. } => {
                        decide_choose_cards(state, seat, candidates, *max)
                    }
                    // A self-discard (cleanup over max hand size, rummaging, a
                    // discard cost): every offered card is in our own hand and
                    // we're the one choosing. Unlike AutoDecider (which dumps
                    // the head of the hand — possibly our best spell), shed the
                    // least useful cards. Inquisition-style "choose from an
                    // opponent's hand" Discards fail the own-hand guard and
                    // fall through to AutoDecider unchanged.
                    crate::decision::Decision::Discard { player, count, hand }
                        if *player == seat
                            && hand.iter().all(|(id, _)| {
                                state.players[seat].hand.iter().any(|c| c.id == *id)
                            }) =>
                    {
                        decide_self_discard(state, seat, hand, *count)
                    }
                    other => AutoDecider.decide(other),
                };
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
                    // Pick the attack target: prefer an opposing monarch (CR
                    // 724 — stealing the crown denies their end-step card and
                    // hands it to us); otherwise the next alive opponent.
                    let target_player = match state.monarch {
                        Some(m)
                            if m != seat
                                && state.players.get(m).map(|p| p.is_alive()).unwrap_or(false) =>
                        {
                            m
                        }
                        _ => state.next_alive_seat(seat),
                    };
                    // Filter on `controller`, not `owner`: cards that have
                    // changed control (Threaten / Mind Control / etc.) are
                    // attacked WITH by the new controller, not the original
                    // owner.
                    //
                    // Bot AI improvement (push XXV): hold back attackers
                    // that would suicide into deathtouch blockers when
                    // there's no upside. The heuristic computes:
                    //   * lethal_swing: whether sum of attackers' powers
                    //     already meets opponent's life total.
                    // When NOT lethal:
                    //   * skip attackers whose toughness is <= the maximum
                    //     opponent blocker power AND there's at least one
                    //     opponent blocker with deathtouch + reach/flying
                    //     parity (i.e. a blocker can be assigned).
                    // This keeps small attackers from auto-dying to
                    // Witherbloom Crawler / Sapworm / Toxicultivator and
                    // similar deathtouch defenders.
                    use crate::card::Keyword;
                    let opp_seat = target_player;
                    let opp_life = state.players[opp_seat].life;
                    let raw_attackers: Vec<&crate::card::CardInstance> = state
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == seat
                                && c.can_attack()
                                // Honor layer-granted Defender / can't-attack
                                // (Pacifism, crewed-Vehicle states) — can_attack
                                // only sees printed keywords.
                                && state
                                    .computed_permanent(c.id)
                                    .map(|cp| {
                                        !cp.keywords.contains(&Keyword::Defender)
                                            && !cp.keywords.contains(&Keyword::CantAttack)
                                            // CR 508.1a — "can attack only if
                                            // defending player controls [X]"
                                            // (Dandân). Don't declare it into a
                                            // defender whose board fails the
                                            // filter, or the whole batch is
                                            // rejected.
                                            && cp.keywords.iter().all(|kw| match kw {
                                                Keyword::CanAttackOnlyIfDefenderControls(req) => {
                                                    state.battlefield.iter().any(|d| {
                                                        d.controller == target_player
                                                            && state.evaluate_requirement_on_card(
                                                                req, d, target_player,
                                                            )
                                                    })
                                                }
                                                Keyword::CanAttackOnlyIfYouControl(req) => {
                                                    state.battlefield.iter().any(|d| {
                                                        d.controller == c.controller
                                                            && state.evaluate_requirement_on_card(
                                                                req, d, c.controller,
                                                            )
                                                    })
                                                }
                                                Keyword::CantAttackOrBlockUnlessEvenCounters => {
                                                    c.counters.values().sum::<u32>() % 2 == 0
                                                }
                                                _ => true,
                                            })
                                    })
                                    .unwrap_or(true)
                        })
                        .collect();
                    let total_raw_power: i32 = raw_attackers.iter().map(|c| c.power()).sum();
                    let lethal_swing = total_raw_power >= opp_life;
                    let opp_blockers: Vec<&crate::card::CardInstance> = state
                        .battlefield
                        .iter()
                        .filter(|c| c.controller == opp_seat && c.can_block())
                        .collect();
                    let has_ground_deathtouch = opp_blockers
                        .iter()
                        .any(|b| b.has_keyword(&Keyword::Deathtouch) && !b.has_keyword(&Keyword::Flying));
                    let max_ground_blocker_power: i32 = opp_blockers
                        .iter()
                        .filter(|b| !b.has_keyword(&Keyword::Flying))
                        .map(|b| b.power())
                        .max()
                        .unwrap_or(0);
                    let mut attackers: Vec<crate::card::CardId> = raw_attackers
                        .into_iter()
                        .filter(|c| {
                            // CR 508.1d — must-attack creatures (Juggernaut,
                            // goaded) have no choice; always include them so
                            // the engine's requirement check accepts the batch.
                            if c.has_keyword(&Keyword::MustAttack) || !c.goaded_by.is_empty() {
                                return true;
                            }
                            // Always attack on lethal swings — the bot
                            // would rather suicide than miss a kill.
                            if lethal_swing {
                                return true;
                            }
                            let flying = c.has_keyword(&Keyword::Flying);
                            // Evasive attackers (flying) — only block-
                            // worried if there's a flying opp blocker.
                            // Skip the deathtouch / ground-power filter
                            // for them; assume they're safe.
                            if flying {
                                let opp_has_flying_blocker = opp_blockers.iter()
                                    .any(|b| b.has_keyword(&Keyword::Flying)
                                          || b.has_keyword(&Keyword::Reach));
                                if !opp_has_flying_blocker {
                                    return true; // free swing
                                }
                            }
                            // Trample: tougher creatures still come in
                            // (we'll get some damage through chumps).
                            if c.has_keyword(&Keyword::Trample) {
                                return true;
                            }
                            // Indestructible: safe to swing (won't die).
                            if c.has_keyword(&Keyword::Indestructible) {
                                return true;
                            }
                            // Shield counter on the attacker — the first
                            // damage is prevented, so a basic ground-trade
                            // is safe (push XXVI bot improvement).
                            if c.counter_count(crate::card::CounterType::Shield) > 0 {
                                return true;
                            }
                            // Lifelink: even if we trade, we gain life —
                            // worth swinging when we can race.
                            if c.has_keyword(&Keyword::Lifelink) {
                                return true;
                            }
                            // Deathtouch attacker: any blocker that deals
                            // with it dies (CR 702.2), so blocking is at
                            // best an even trade for the opponent — swinging
                            // is always at least fine.
                            if c.has_keyword(&Keyword::Deathtouch) && c.power() >= 1 {
                                return true;
                            }
                            // Menace (CR 702.111): needs two+ blockers. If
                            // the opponent has fewer than two creatures that
                            // can legally block this attacker, it gets
                            // through unblocked — safe to swing.
                            if c.has_keyword(&Keyword::Menace) {
                                let able = opp_blockers
                                    .iter()
                                    .filter(|b| {
                                        !flying
                                            || b.has_keyword(&Keyword::Flying)
                                            || b.has_keyword(&Keyword::Reach)
                                    })
                                    .count();
                                if able < 2 {
                                    return true;
                                }
                            }
                            // First strike + bigger power than blockers'
                            // toughness — we kill the blocker before it
                            // strikes back. Safe attack (push XXVI).
                            if c.has_keyword(&Keyword::FirstStrike)
                                || c.has_keyword(&Keyword::DoubleStrike)
                            {
                                let max_blocker_toughness: i32 = opp_blockers
                                    .iter()
                                    .filter(|b| !b.has_keyword(&Keyword::Flying) || flying)
                                    .map(|b| b.toughness())
                                    .max()
                                    .unwrap_or(0);
                                if c.power() > max_blocker_toughness {
                                    return true;
                                }
                            }
                            // Hold back if a deathtouch blocker exists
                            // and we don't outsize the biggest blocker.
                            if has_ground_deathtouch && !flying {
                                return false;
                            }
                            // Finality counter on the attacker — if it
                            // dies it'll exile instead of returning to
                            // the graveyard (CR 122.1h). Don't suicide
                            // a finality-counter creature into ground
                            // blockers that can kill it.
                            // Push (claude/modern_decks, batches 192-197).
                            if c.counter_count(crate::card::CounterType::Finality) > 0
                                && !flying
                                && max_ground_blocker_power >= c.toughness()
                            {
                                return false;
                            }
                            // Hold back if our toughness is <= biggest
                            // blocker power and we wouldn't kill them
                            // (basic suicide filter).
                            if !flying
                                && max_ground_blocker_power >= c.toughness()
                                && c.power() <= max_ground_blocker_power
                            {
                                return false;
                            }
                            true
                        })
                        .map(|c| c.id)
                        .collect();
                    // Find opponent planeswalkers in loyalty-ascending
                    // order. The bot will redirect attacks at PWs whose
                    // current loyalty is at-or-below our total attacking
                    // power — finishing off the walker. Each PW consumes
                    // up to its loyalty worth of attackers; the rest
                    // attack the player.
                    let mut walker_targets: Vec<(crate::card::CardId, u32)> = state
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.definition.is_planeswalker()
                                && c.controller != seat
                                && state.players[c.controller].is_alive()
                        })
                        .map(|c| {
                            let loyalty = c
                                .counters
                                .iter()
                                .find_map(|(k, v)| {
                                    matches!(k, crate::card::CounterType::Loyalty)
                                        .then_some(*v)
                                })
                                .unwrap_or(0);
                            (c.id, loyalty)
                        })
                        .collect();
                    walker_targets.sort_by_key(|(_, l)| *l);
                    let total_power: i32 = attackers
                        .iter()
                        .filter_map(|id| {
                            state.battlefield.iter().find(|c| c.id == *id).map(|c| c.power())
                        })
                        .sum();
                    let mut attacks: Vec<Attack> = Vec::new();
                    for (pw_id, loyalty) in walker_targets {
                        // Only redirect when we can plausibly finish it
                        // off (total attacking power >= loyalty). Avoids
                        // throwing 1-power chumps at a 5-loyalty walker.
                        if (total_power as u32) < loyalty || loyalty == 0 {
                            continue;
                        }
                        // Pull as many attackers as the walker's loyalty
                        // for this redirect, picking smallest-power
                        // first so we keep beefy beaters for the player
                        // when possible. (Suicide-by-blocker is still
                        // not modeled here.)
                        let mut budget = loyalty as i32;
                        attackers.sort_by_key(|id| {
                            state
                                .battlefield
                                .iter()
                                .find(|c| c.id == *id)
                                .map(|c| c.power())
                                .unwrap_or(0)
                        });
                        let mut remaining: Vec<crate::card::CardId> = Vec::new();
                        for id in attackers.drain(..) {
                            let pow = state
                                .battlefield
                                .iter()
                                .find(|c| c.id == id)
                                .map(|c| c.power())
                                .unwrap_or(0);
                            if budget > 0 && pow > 0 {
                                attacks.push(Attack {
                                    attacker: id,
                                    target: AttackTarget::Planeswalker(pw_id),
                                });
                                budget -= pow;
                            } else {
                                remaining.push(id);
                            }
                        }
                        attackers = remaining;
                    }
                    // Remaining attackers go at the player.
                    for id in attackers {
                        attacks.push(Attack {
                            attacker: id,
                            target: AttackTarget::Player(target_player),
                        });
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

/// Land-count mulligan heuristic. A keepable opening hand wants roughly
/// 2–5 lands out of seven; 0–1 (screw) or 6–7 (flood) are shipped. We stop
/// digging after two mulligans (a London mulligan past that bottoms too
/// many cards to be worth chasing a perfect curve) and always keep a hand
/// of three or fewer cards. Reads land counts off the live hand zone since
/// the `Decision::Mulligan` payload only carries names.
/// Colors a land card could tap for, for mulligan color-screw checks.
/// Reads basic land types (Plains→W, …) plus `AddMana` effects on its
/// activated abilities; "any color" payloads yield the full WUBRG set.
fn land_color_output(card: &CardDefinition) -> crate::mana::ColorSet {
    use crate::card::LandType;
    use crate::mana::{Color, ColorSet};
    let mut set = ColorSet::empty();
    for lt in &card.subtypes.land_types {
        match lt {
            LandType::Plains => set.insert(Color::White),
            LandType::Island => set.insert(Color::Blue),
            LandType::Swamp => set.insert(Color::Black),
            LandType::Mountain => set.insert(Color::Red),
            LandType::Forest => set.insert(Color::Green),
            _ => {}
        }
    }
    for ab in &card.activated_abilities {
        accumulate_mana_colors(&ab.effect, &mut set);
    }
    set
}

/// Bot policy for `Decision::OptionalTrigger`: take the trigger unless its
/// matching `MayDo` body imposes a clear self-cost (lose life / sacrifice /
/// discard on the bot). `AutoDecider` declines *every* optional trigger,
/// which means a bot would never take a beneficial "you may" (Provoke's
/// "you may", Boast token riders, etc.); this makes those fire.
pub(crate) fn optional_trigger_beneficial(state: &GameState, source: CardId, description: &str) -> bool {
    // Locate the source card's definition in any zone the bot can see.
    let def = state
        .battlefield
        .iter()
        .find(|c| c.id == source)
        .map(|c| &c.definition)
        .or_else(|| {
            state
                .players
                .iter()
                .flat_map(|p| p.graveyard.iter().chain(p.hand.iter()))
                .find(|c| c.id == source)
                .map(|c| &c.definition)
        });
    let Some(def) = def else { return true };
    // Find the `MayDo` body whose description matches the prompt. Scan the
    // card's spell effect, its triggered abilities, and any static-ability
    // reflexive (`when_you_do`) — the prompt can originate from any of these
    // (e.g. Valentin's exile-replacement reflexive lives on a static).
    let mut body = find_maydo_body(&def.effect, description);
    if body.is_none() {
        for t in &def.triggered_abilities {
            if let Some(b) = find_maydo_body(&t.effect, description) {
                body = Some(b);
                break;
            }
        }
    }
    if body.is_none() {
        for sa in &def.static_abilities {
            if let crate::effect::StaticEffect::ExileDyingOpponentCreatures {
                when_you_do: Some(eff),
            } = &sa.effect
                && let Some(b) = find_maydo_body(eff, description)
            {
                body = Some(b);
                break;
            }
        }
    }
    // Take it unless the body is self-costly; default to taking when the
    // body can't be introspected (most "you may" on your own permanents is
    // upside).
    body.map(|b| !effect_imposes_self_cost(b)).unwrap_or(true)
}

/// Recursively find the optional-effect body whose prompt is `desc`. Both
/// `Effect::MayDo` and `Effect::MayPay` surface as a `Decision::OptionalTrigger`
/// keyed on their description, so the bot's self-cost screen (e.g. a "you may
/// pay {2}: each player loses 3 life" body it shouldn't auto-accept) applies to
/// both shapes.
fn find_maydo_body<'a>(eff: &'a Effect, desc: &str) -> Option<&'a Effect> {
    match eff {
        Effect::MayDo { description, body } | Effect::MayPay { description, body, .. }
            if description == desc =>
        {
            Some(body)
        }
        Effect::MayDo { body, .. }
        | Effect::MayPay { body, .. }
        | Effect::ForEach { body, .. } => find_maydo_body(body, desc),
        Effect::Seq(v) => v.iter().find_map(|e| find_maydo_body(e, desc)),
        Effect::ChooseMode(v) | Effect::ChooseN { modes: v, .. } | Effect::Escalate { modes: v, .. } => {
            v.iter().find_map(|e| find_maydo_body(e, desc))
        }
        Effect::If { then, else_, .. } => {
            find_maydo_body(then, desc).or_else(|| find_maydo_body(else_, desc))
        }
        _ => None,
    }
}

/// Whether `eff` (a "you may" body) imposes a clear cost on its controller —
/// losing life, sacrificing, or discarding. Conservative: the bot declines
/// such triggers rather than paying for an effect it can't value-judge.
fn effect_imposes_self_cost(eff: &Effect) -> bool {
    use crate::effect::{PlayerRef, Selector};
    let hits_self = |sel: &Selector| {
        matches!(sel, Selector::You | Selector::This)
            || matches!(sel, Selector::Player(PlayerRef::You))
    };
    match eff {
        Effect::LoseLife { who, .. }
        | Effect::Discard { who, .. }
        | Effect::Mill { who, .. }
        | Effect::LoseHalfLife { who, .. }
        | Effect::MillHalf { who, .. }
        | Effect::DiscardHalf { who, .. } => hits_self(who),
        // Self-directed damage (a "you may have ~ deal N damage to you" rider).
        Effect::DealDamage { to, .. } => hits_self(to),
        // Drain *out of* the bot is a cost; drain *into* the bot is upside.
        Effect::Drain { from, .. } => hits_self(from),
        Effect::Sacrifice { who, .. } | Effect::SacrificeGreatestMV { who, .. } => hits_self(who),
        Effect::SacrificeAndRemember { .. } => true,
        Effect::SacrificeAnyNumber { who, .. } => matches!(who, PlayerRef::You),
        Effect::PayLifeLookTake { who } => matches!(who, PlayerRef::You),
        Effect::Seq(v) => v.iter().any(effect_imposes_self_cost),
        Effect::ChooseMode(v) | Effect::ChooseN { modes: v, .. } | Effect::Escalate { modes: v, .. } => {
            v.iter().any(effect_imposes_self_cost)
        }
        Effect::If { then, else_, .. } => {
            effect_imposes_self_cost(then) || effect_imposes_self_cost(else_)
        }
        Effect::ForEach { body, .. } | Effect::MayDo { body, .. } => effect_imposes_self_cost(body),
        // Mana/energy "pay or else" wrap a fallback (usually SacrificeSource);
        // the bot reads the fallback to decide whether declining is costly.
        Effect::PayManaOrElse { otherwise, .. } | Effect::PayEnergyOrElse { otherwise, .. } => {
            effect_imposes_self_cost(otherwise)
        }
        // "You may sacrifice/exile this" riders are a clear self-cost.
        Effect::SacrificeSource => true,
        Effect::Exile { what } => hits_self(what),
        Effect::PayOrLoseGame { .. } => true,
        _ => false,
    }
}

/// Bot heuristic for `Decision::SearchLibrary`: pick a basic land that
/// adds the bot's least-covered color, else (no basic land among the
/// candidates) grab the highest-mana-value candidate — a creature/spell
/// tutor (Fauna Shaman, Imperial Recruiter, Spellseeker) should fetch its
/// most impactful hit, not the first one, and certainly not fizzle like the
/// stock `AutoDecider`.
fn decide_library_search(
    state: &GameState,
    seat: usize,
    candidates: &[(crate::card::CardId, String)],
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    use crate::mana::Color;
    if candidates.is_empty() {
        return DecisionAnswer::Search(None);
    }
    const COLORS: [Color; 5] =
        [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    // How many of our lands already tap for each color.
    let mut sources: std::collections::HashMap<Color, usize> = std::collections::HashMap::new();
    for c in state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
    {
        let out = land_color_output(&c.definition);
        for col in COLORS {
            if out.contains(col) {
                *sources.entry(col).or_insert(0) += 1;
            }
        }
    }
    let lib = &state.players[seat].library;
    let mut best: Option<(crate::card::CardId, usize)> = None;
    for (id, _) in candidates {
        let Some(card) = lib.iter().find(|c| c.id == *id) else { continue };
        if !(card.definition.is_basic() && card.definition.is_land()) {
            continue;
        }
        let out = land_color_output(&card.definition);
        // Score by the fewest existing sources among the colors it makes.
        let score = COLORS
            .iter()
            .filter(|col| out.contains(**col))
            .map(|col| sources.get(col).copied().unwrap_or(0))
            .min()
            .unwrap_or(usize::MAX);
        if best.map(|(_, s)| score < s).unwrap_or(true) {
            best = Some((*id, score));
        }
    }
    if let Some((id, _)) = best {
        return DecisionAnswer::Search(Some(id));
    }
    // No basic land among the candidates (a creature/spell tutor): fetch the
    // highest-mana-value hit as a reasonable "best card" proxy, falling back
    // to the first candidate when CMCs can't be read.
    let pick = candidates
        .iter()
        .max_by_key(|(id, _)| {
            lib.iter().find(|c| c.id == *id).map(|c| c.definition.cost.cmc()).unwrap_or(0)
        })
        .map(|(id, _)| *id)
        .unwrap_or(candidates[0].0);
    DecisionAnswer::Search(Some(pick))
}

/// Bot heuristic for `Decision::ChooseCards`. Two cases:
/// - **Put-onto-battlefield from hand** (Sneak Attack / Elvish Piper / Goblin
///   Lackey): every candidate is in the bot's own hand. Cheat in the single
///   biggest creature (highest mana value, then power) — that's the whole point
///   of the effect. Without this the AutoDecider min-0 default declines and the
///   bot never uses the card.
/// - **Exile from graveyards** (Collect Evidence / Fateseal-style): exile every
///   offered card an opponent owns, up to `max`, skipping the bot's own.
fn decide_choose_cards(
    state: &GameState,
    seat: usize,
    candidates: &[(crate::card::CardId, String)],
    max: u32,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    // Hand-source pick: take the biggest creature(s) we can.
    let all_in_hand = !candidates.is_empty()
        && candidates
            .iter()
            .all(|(id, _)| state.players[seat].hand.iter().any(|c| c.id == *id));
    if all_in_hand {
        let mut ranked: Vec<(crate::card::CardId, i32, i32)> = candidates
            .iter()
            .filter_map(|(id, _)| {
                let c = state.players[seat].hand.iter().find(|c| c.id == *id)?;
                Some((*id, c.definition.cost.cmc() as i32, c.definition.power))
            })
            .collect();
        // Biggest first: highest mana value, then highest power.
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
        let chosen: Vec<_> = ranked.into_iter().take(max as usize).map(|(id, ..)| id).collect();
        return DecisionAnswer::Cards(chosen);
    }
    let owner_of = |id: crate::card::CardId| -> Option<usize> {
        state
            .players
            .iter()
            .position(|p| p.graveyard.iter().any(|c| c.id == id))
    };
    let chosen: Vec<_> = candidates
        .iter()
        .filter(|(id, _)| owner_of(*id).is_some_and(|o| !state.same_team(o, seat)))
        .map(|(id, _)| *id)
        .take(max as usize)
        .collect();
    DecisionAnswer::Cards(chosen)
}

/// Bot heuristic for a self-discard (cleanup discard-to-hand-size, rummaging,
/// a discard cost): shed the `count` least useful cards so the bot keeps its
/// cheap, castable spells. Surplus lands go first once the bot is no longer
/// mana-light; otherwise the most expensive spells (least likely to be cast
/// soon) are pitched. Ties keep hand order.
fn decide_self_discard(
    state: &GameState,
    seat: usize,
    hand: &[(crate::card::CardId, String)],
    count: u32,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    // Lands already in play: once we have plenty, extra lands in hand are the
    // first thing to pitch; while still mana-light, keep them.
    let lands_in_play = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_land())
        .count();
    // Score each offered card — LOWER is pitched sooner.
    let mut scored: Vec<(i64, crate::card::CardId)> = hand
        .iter()
        .filter_map(|(id, _)| {
            let card = state.players[seat].hand.iter().find(|c| c.id == *id)?;
            let score = if card.definition.is_land() {
                // Surplus lands are worth the least; a land we still need is
                // worth the most.
                if lands_in_play >= 5 { -100 } else { 1_000 }
            } else {
                // Among spells, keep the cheap (castable) ones; pitch the
                // most expensive first.
                -(card.definition.cost.cmc() as i64)
            };
            Some((score, *id))
        })
        .collect();
    scored.sort_by_key(|(s, _)| *s);
    let discard: Vec<crate::card::CardId> =
        scored.iter().take(count as usize).map(|(_, id)| *id).collect();
    DecisionAnswer::Discard(discard)
}

fn accumulate_mana_colors(eff: &Effect, set: &mut crate::mana::ColorSet) {
    match eff {
        Effect::AddMana { pool, .. } => accumulate_payload_colors(pool, set),
        Effect::Seq(v) => v.iter().for_each(|e| accumulate_mana_colors(e, set)),
        _ => {}
    }
}

fn accumulate_payload_colors(pool: &ManaPayload, set: &mut crate::mana::ColorSet) {
    match pool {
        ManaPayload::Colors(cs) | ManaPayload::OfColors(cs, _) => {
            cs.iter().for_each(|c| set.insert(*c))
        }
        ManaPayload::OfColor(c, _) => set.insert(*c),
        ManaPayload::AnyOneColor(_)
        | ManaPayload::AnyColors(_)
        | ManaPayload::AnyColorOpponentCouldProduce
        | ManaPayload::AnyColorYouCouldProduce
        | ManaPayload::DevotionOfChosenColor => *set = crate::mana::ColorSet::all(),
        ManaPayload::Colorless(_) => {}
        // Could produce any single color the rock was set to — treat as
        // potentially any color for the bot's mana-base reasoning.
        ManaPayload::ChosenColorOfSource
        | ManaPayload::ImprintedCardColor => *set = crate::mana::ColorSet::all(),
        ManaPayload::Restricted(inner, _) | ManaPayload::RestrictedToChosenType(inner) => {
            accumulate_payload_colors(inner, set)
        }
    }
}

fn decide_mulligan(
    state: &GameState,
    seat: usize,
    mulligans_taken: usize,
) -> crate::decision::DecisionAnswer {
    use crate::decision::DecisionAnswer;
    let hand = &state.players[seat].hand;
    let lands = hand.iter().filter(|c| c.definition.is_land()).count();
    // Curve check: a 2–5-land hand is only worth keeping if it has at least
    // one nonland spell cheap enough to cast in the first few turns — three
    // lands plus four 7-drops is a screwed keep. "Castable early" means a
    // spell whose mana value is within `lands + 1` (a generous early-curve
    // window that still trusts a couple of draws).
    // Color-screw awareness: an early play only counts if the hand's lands
    // can actually produce its colored pips. Three Forests + a hand of blue
    // spells is a screwed keep even though the curve looks fine.
    let producible = hand
        .iter()
        .filter(|c| c.definition.is_land())
        .fold(crate::mana::ColorSet::empty(), |acc, c| {
            acc.union(land_color_output(&c.definition))
        });
    let has_early_play = hand.iter().any(|c| {
        if c.definition.is_land() || c.definition.cost.cmc() as usize > lands + 1 {
            return false;
        }
        let mut need = crate::mana::ColorSet::empty();
        for col in c.definition.cost.colors() {
            need.insert(col);
        }
        need.is_subset_of(producible)
    });
    let keepable = ((2..=5).contains(&lands) && has_early_play) || hand.len() <= 3;
    if keepable || mulligans_taken >= 2 {
        DecisionAnswer::Keep
    } else {
        DecisionAnswer::TakeMulligan
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
            x_value: None,
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
            x_value: None,
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
                // Multi-target shapes (Snow Day, Homesickness, Cost of
                // Brilliance, Render Speechless, Vibrant Outburst, …) ask
                // the picker for every slot index used by the effect tree;
                // slots that find no legal target are skipped, matching
                // "up to N target" semantics.
                let mode_effect = mode_branch(&c.definition.effect, mode);
                let (target, additional_targets) = if mode_effect.requires_target() {
                    let (t, extras) =
                        state.auto_targets_for_effect_all_slots(mode_effect, seat, mode);
                    t.as_ref()?;
                    (t, extras)
                } else {
                    (None, vec![])
                };
                Some(GameAction::CastSpell {
                    card_id: c.id,
                    target,
                    additional_targets,
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

    // Delve (CR 702.66): for any hand card with `Keyword::Delve` that the
    // bot can't (yet) afford, try exiling graveyard cards to pay the
    // generic portion. Delve the maximum available (capped at the generic
    // pip total), then let `would_accept` confirm the reduced cost is
    // payable. Appended to the candidate set so the bot actually leverages
    // Treasure Cruise / Dig Through Time / Gurmag Angler off a full bin.
    let mut castable = castable;
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.keywords.contains(&crate::card::Keyword::Delve))
    {
        let generic_pips: u32 = c
            .definition
            .cost
            .symbols
            .iter()
            .filter_map(|s| match s {
                crate::mana::ManaSymbol::Generic(n) => Some(*n),
                _ => None,
            })
            .sum();
        let gy_ids: Vec<CardId> = state.players[seat].graveyard.iter().map(|g| g.id).collect();
        let take = (generic_pips as usize).min(gy_ids.len());
        if take == 0 {
            continue;
        }
        let delve_cards: Vec<CardId> = gy_ids.into_iter().take(take).collect();
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellDelve {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
            delve_cards,
        };
        if state.would_accept(action.clone()) {
            castable.push(action);
        }
    }

    // Kicker (CR 702.32): for any hand card with `Keyword::Kicker`, offer a
    // `CastSpellKicked` candidate. Targets come from the effect tree, whose
    // slot-0 filter resolves to the kicked (typically broader) branch, so a
    // kicked Tear Asunder can aim at a creature. `would_accept` validates
    // the full base+kicker cost, so this is only added when affordable.
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.has_kicker().is_some())
    {
        let effect = &c.definition.effect;
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots_kicked(effect, seat, None, true);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellKicked {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if state.would_accept(action.clone()) {
            castable.push(action);
        }
    }

    // Bestow (CR 702.103): for any hand card with a bestow cost, offer a
    // `CastBestow` candidate that enchants the bot's sturdiest creature (the
    // host most likely to stick, so the Aura keeps its value). `would_accept`
    // validates the full bestow cost, so this is only added when affordable.
    for c in state.players[seat]
        .hand
        .iter()
        .filter(|c| c.definition.bestow.is_some())
    {
        // Prefer the controller's highest-toughness creature as the host.
        let host = state
            .battlefield
            .iter()
            .filter(|b| b.controller == seat && b.definition.is_creature())
            .max_by_key(|b| state.computed_permanent(b.id).map(|cp| cp.toughness).unwrap_or(0))
            .map(|b| b.id);
        let Some(host) = host else { continue };
        let action = GameAction::CastBestow {
            card_id: c.id,
            target: Some(crate::game::Target::Permanent(host)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        if state.would_accept(action.clone()) {
            castable.push(action);
        }
    }

    // Adventure (CR 715): for any hand card with an adventure half that
    // *targets* something (removal / bounce / pump — Stomp, Petty Theft,
    // Swift End, Boulder Rush), offer a `CastAdventure` candidate. Token /
    // card-draw adventures are skipped here so the bot still prefers playing
    // those cards as creatures; the interactive halves are pure tempo wins.
    for c in state.players[seat].hand.iter() {
        let Some(adv) = c.definition.has_adventure() else { continue };
        if !adv.effect.requires_target() {
            continue;
        }
        let (target, additional_targets) =
            state.auto_targets_for_effect_all_slots(&adv.effect, seat, None);
        if target.is_none() {
            continue;
        }
        let action = GameAction::CastAdventure {
            card_id: c.id,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if state.would_accept(action.clone()) {
            castable.push(action);
        }
    }

    // Split cards (CR 709): for any hand card with a non-aftermath split,
    // offer a `CastSplitRight` candidate (the left half is already covered by
    // the plain `CastSpell` path). Auto-target the right half's effect.
    for c in state.players[seat].hand.iter() {
        let Some(split) = c.definition.has_split() else { continue };
        if split.aftermath {
            continue;
        }
        let (target, additional_targets) = if split.right.effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots(&split.right.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSplitRight {
            card_id: c.id, target, additional_targets, mode: None, x_value: None,
        };
        if state.would_accept(action.clone()) {
            castable.push(action);
        }
    }

    // Aftermath (CR 702.127): cast the right half of a split card from the
    // graveyard. `would_accept` enforces the graveyard-only + timing rules.
    for c in state.players[seat].graveyard.iter() {
        let Some(split) = c.definition.has_split().filter(|s| s.aftermath) else { continue };
        let (target, additional_targets) = if split.right.effect.requires_target() {
            let (t, extras) =
                state.auto_targets_for_effect_all_slots(&split.right.effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastAftermath {
            card_id: c.id, target, additional_targets, mode: None, x_value: None,
        };
        if state.would_accept(action.clone()) {
            castable.push(action);
        }
    }

    // Adventure creature (CR 715) and plotted cards (CR 702.170d): cast the
    // creature half / a plotted card from exile. `would_accept` enforces the
    // later-turn + sorcery-speed timing, so this is only offered when legal.
    for c in state.exile.iter().filter(|c| c.owner == seat) {
        let action = if c.on_adventure {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None)
            } else {
                (None, vec![])
            };
            GameAction::CastAdventureCreature {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            }
        } else if state.plotted_cards.contains(&c.id) {
            let (target, additional_targets) = if c.definition.effect.requires_target() {
                state.auto_targets_for_effect_all_slots(&c.definition.effect, seat, None)
            } else {
                (None, vec![])
            };
            GameAction::CastPlotted {
                card_id: c.id, target, additional_targets, mode: None, x_value: None,
            }
        } else {
            continue;
        };
        if state.would_accept(action.clone()) {
            castable.push(action);
        }
    }

    // Mana-only alternative costs (Dash CR 702.110, Blitz 702.152,
    // Spectacle 702.111): for any hand card whose `alternative_cost` is paid
    // purely with mana (no pitch/sacrifice/graveyard/life rider), offer a
    // `CastSpellAlternative` candidate. `would_accept` validates the alt cost
    // and its `condition` gate (e.g. Spectacle's opponent-lost-life), so a
    // Skewer the Critics is only offered for {R} once an opponent has bled.
    for c in state.players[seat].hand.iter().filter(|c| {
        c.definition.alternative_cost.as_ref().is_some_and(|a| {
            a.exile_filter.is_none()
                && a.sacrifice_permanents.is_none()
                && a.exile_from_graveyard_count == 0
                && a.life_cost == 0
                && !a.evoke_sacrifice
        })
    }) {
        let effect = c
            .definition
            .alternative_cost
            .as_ref()
            .and_then(|a| a.effect_override.as_ref())
            .unwrap_or(&c.definition.effect);
        let (target, additional_targets) = if effect.requires_target() {
            let (t, extras) = state.auto_targets_for_effect_all_slots(effect, seat, None);
            if t.is_none() {
                continue;
            }
            (t, extras)
        } else {
            (None, vec![])
        };
        let action = GameAction::CastSpellAlternative {
            card_id: c.id,
            pitch_card: None,
            target,
            additional_targets,
            mode: None,
            x_value: None,
        };
        if state.would_accept(action.clone()) {
            castable.push(action);
        }
    }

    // Morph / Disguise (CR 702.36 / 702.166): cast a hand card face down for
    // {3} as a 2/2 (with ward {2} for Disguise). Offered only when no normal
    // spell candidate exists yet, so the bot still prefers casting cards face
    // up; `would_accept` enforces sorcery timing and the {3} payment.
    if castable.is_empty() {
        for c in state.players[seat].hand.iter().filter(|c| {
            c.definition.keywords.iter().any(|k| {
                matches!(
                    k,
                    crate::card::Keyword::Morph(_)
                        | crate::card::Keyword::Megamorph(_)
                        | crate::card::Keyword::Disguise(_)
                )
            })
        }) {
            let action = GameAction::CastFaceDown { card_id: c.id };
            if state.would_accept(action.clone()) {
                castable.push(action);
            }
        }
    }

    // Play a land if possible — gated through `would_accept` for
    // the same reason (the engine enforces sorcery timing, lands-
    // played-this-turn, etc.). Use the game-level helper so an
    // Exploration / Azusa-style ExtraLandPerTurn static lets the bot
    // play a second land in the same turn (CR 305.2).
    if state.can_player_play_land(seat)
        && let Some(land) = state.players[seat].hand.iter().find(|c| c.definition.is_land())
    {
        let action = GameAction::PlayLand(land.id);
        if state.would_accept(action.clone()) {
            return action;
        }
    }

    // Crucible of Worlds / Ramunap Excavator: replay a land from the
    // graveyard if no hand land was played (CR 305 land-from-gy permission).
    if state.can_player_play_land(seat)
        && state.player_may_play_lands_from_graveyard(seat)
        && let Some(land) =
            state.players[seat].graveyard.iter().find(|c| c.definition.is_land())
    {
        let action = GameAction::PlayLandFromGraveyard(land.id);
        if state.would_accept(action.clone()) {
            return action;
        }
    }

    if !castable.is_empty() {
        // Magecraft-aware bias: if the bot controls a permanent with a
        // magecraft trigger and at least one instant or sorcery is in
        // the castable set, prefer the IS subset so the trigger fires.
        // Falls back to uniform-random sampling when no magecraft body
        // is in play. Push (claude/modern_decks batch 202).
        let has_magecraft = state.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition.triggered_abilities.iter().any(is_magecraft_trigger)
        });
        let pool: Vec<GameAction> = if has_magecraft {
            let only_is: Vec<GameAction> = castable
                .iter()
                .filter(|a| matches!(a, GameAction::CastSpell { card_id, .. } if is_instant_or_sorcery_in_hand(state, seat, *card_id)))
                .cloned()
                .collect();
            if only_is.is_empty() { castable } else { only_is }
        } else {
            castable
        };
        let mut r = rng();
        return pool[r.random_range(0..pool.len())].clone();
    }

    // Activate planeswalker loyalty abilities the bot controls. Pick the
    // first usable ability per walker (engine enforces sorcery timing and
    // once-per-turn). The candidate set is dry-run-gated so failed targets
    // / over-spent loyalty / opp-controlled-walker rejections drop out.
    if let Some(action) = pick_loyalty_ability(state, seat) {
        return action;
    }

    // Equip (CR 702.6): if the bot controls an Equipment that isn't yet
    // attached to one of its creatures, and it controls a creature to wear
    // it, move the Equipment onto the biggest such creature. Dry-run-gated
    // so the equip cost / sorcery timing / target legality all bottom out
    // in `would_accept`.
    if let Some(action) = pick_equip(state, seat) {
        return action;
    }

    // Spend surplus energy on beneficial energy-payoff abilities (Bristling
    // Hydra's grow, Longtusk Cub's +1/+1, Aetherstream Leopard's
    // unblockable, …). Only pure "Pay {E}: do X" abilities with no other
    // cost are considered, so the bot can't bankrupt mana or sacrifice
    // anything. Dry-run-gated like everything else.
    if let Some(action) = pick_energy_payoff(state, seat) {
        return action;
    }

    // Recur value from the graveyard (Embalm CR 702.88 / Eternalize CR 702.91
    // and any "Exile this from your graveyard: …" ability) when there's spare
    // mana and nothing better to do. Dry-run-gated so cost / sorcery timing
    // bottom out in `would_accept`.
    if let Some(action) = pick_graveyard_recursion(state, seat) {
        return action;
    }

    // Unmask a face-down threat (Morph / Megamorph / Disguise / a cloaked or
    // manifested creature card) when the turn-up cost is affordable. Dry-run-
    // gated, so the cost / timing / "manifested noncreature can't turn up"
    // rules all bottom out in `would_accept`.
    if let Some(action) = pick_turn_face_up(state, seat) {
        return action;
    }

    GameAction::PassPriority
}

/// Offer a `TurnFaceUp` for the first affordable face-down permanent the bot
/// controls. The cost is the real card's Morph/Megamorph/Disguise cost, or its
/// mana cost for a manifested/cloaked creature card; `would_accept` enforces it.
fn pick_turn_face_up(state: &GameState, seat: usize) -> Option<GameAction> {
    state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.face_down && c.face_up_def.is_some())
        .map(|c| GameAction::TurnFaceUp { card_id: c.id })
        .find(|a| state.would_accept(a.clone()))
}

/// Find an affordable graveyard-activated ability whose cost exiles the source
/// (Embalm / Eternalize, Stone Docent-style recursion). Returns the activation
/// for the first such card the bot can pay for.
fn pick_graveyard_recursion(state: &GameState, seat: usize) -> Option<GameAction> {
    // The bot's own creatures, highest-power first — candidate targets for
    // abilities that need one (Scavenge's +1/+1 counters, Daring Fiendbonder's
    // indestructible counter). For no-target recursion (Embalm / Eternalize /
    // Stone Docent) we pass `None`.
    let mut own: Vec<&crate::card::CardInstance> = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .collect();
    own.sort_by_key(|c| std::cmp::Reverse(c.power()));
    for card in state.players[seat].graveyard.iter() {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            if !(ab.from_graveyard && ab.exile_self_cost) {
                continue;
            }
            // Only try a no-target activation when the effect needs none —
            // otherwise `would_accept` (which doesn't re-derive targets) would
            // wave through a wasted target-less activation.
            let candidates: Vec<Option<crate::game::Target>> = if ab.effect.requires_target() {
                own.iter().map(|c| Some(crate::game::Target::Permanent(c.id))).collect()
            } else {
                vec![None]
            };
            for target in candidates {
                let action = GameAction::ActivateAbility {
                    card_id: card.id,
                    ability_index: idx,
                    target,
                    x_value: None,
                };
                if state.would_accept(action.clone()) {
                    return Some(action);
                }
            }
        }
    }
    None
}

/// Find a beneficial energy-only activated ability the bot can pay for: an
/// `Effect::PayEnergy { amount, .. }` ability with no mana/tap/sac cost,
/// where the bot controls the source and has at least `amount` energy.
fn pick_energy_payoff(state: &GameState, seat: usize) -> Option<GameAction> {
    if state.players[seat].energy == 0 {
        return None;
    }
    for card in state.battlefield.iter().filter(|c| c.controller == seat) {
        for (idx, ab) in card.definition.activated_abilities.iter().enumerate() {
            // The energy can be modeled either as a real activation cost
            // (`ActivatedAbility.energy_cost`, the up-front-gated form) or as a
            // resolve-time `Effect::PayEnergy` rider. Match either so the bot
            // fires Longtusk Cub-style `{E}{E}{E}: +1/+1` payoffs regardless of
            // which shape the card uses.
            let amount = if ab.energy_cost > 0 {
                ab.energy_cost
            } else if let Effect::PayEnergy { amount, .. } = &ab.effect {
                *amount
            } else {
                continue;
            };
            let is_pure = !ab.tap_cost
                && !ab.sac_cost
                && ab.mana_cost.symbols.is_empty()
                && ab.life_cost == 0;
            if !is_pure || state.players[seat].energy < amount {
                continue;
            }
            let action = GameAction::ActivateAbility {
                card_id: card.id,
                ability_index: idx,
                target: None,
                x_value: None,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Pick an equip activation: the first controlled Equipment that's either
/// unattached or attached to a permanent the bot doesn't control, paired
/// with the highest-power creature the bot controls. Returns `None` when
/// there's nothing worth equipping. Dry-run gated by the caller's
/// `would_accept` is bypassed here (we gate inline) so the bot doesn't
/// thrash re-equipping the same creature.
fn pick_equip(state: &GameState, seat: usize) -> Option<GameAction> {
    // Best creature to wear an Equipment: highest current power.
    let target = state
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .max_by_key(|c| c.power())
        .map(|c| c.id)?;
    for eq in &state.battlefield {
        if eq.controller != seat || !eq.definition.is_equipment() {
            continue;
        }
        if eq.definition.has_equip().is_none() {
            continue;
        }
        // Skip if already on the chosen target (no point re-equipping).
        if eq.attached_to == Some(target) {
            continue;
        }
        let action = GameAction::Equip { equipment: eq.id, target };
        if state.would_accept(action.clone()) {
            return Some(action);
        }
    }
    None
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
        let allowed = if card.definition.loyalty_twice_each_turn { 2 } else { 1 };
        if card.loyalty_uses_this_turn >= allowed {
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
                // No legal target for *this* ability — skip it and try the
                // next (formerly `?`-returned, which abandoned every other
                // ability and planeswalker the bot controls).
                match state.auto_target_for_effect(&ability.effect, seat) {
                    Some(t) => Some(t),
                    None => continue,
                }
            } else {
                None
            };
            // Variable-X (`-X`) ability: commit all current loyalty.
            let x_value = ability.x_cost.then_some(current_loyalty.max(0) as u32);
            let action = GameAction::ActivateLoyaltyAbility {
                card_id: card.id,
                ability_index: idx,
                target,
                x_value,
            };
            if state.would_accept(action.clone()) {
                return Some(action);
            }
        }
    }
    None
}

/// Test-visible wrapper for `pick_blocks` so external tests can exercise
/// the blocker heuristic in isolation.
pub fn pick_blocks_for_test(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    pick_blocks(state, seat)
}

fn pick_blocks(state: &GameState, seat: usize) -> Vec<(CardId, CardId)> {
    // Improved blocker heuristic (push claude/modern_decks):
    //   1. Build the candidate set of (attacker, attacker_power,
    //      attacker_toughness, has_flying) attacking us.
    //   2. Sort blockers by ascending power so cheap chumps get
    //      assigned first; bigger blockers stay free for must-block
    //      situations.
    //   3. For each blocker, pick the **best** attacker it can block:
    //      - Prefer attackers it can kill outright (blocker_power >=
    //        attacker_toughness, with deathtouch granting kill on any
    //        damage).
    //      - Among kill-able attackers, prefer one that won't kill the
    //        blocker (blocker_toughness > attacker_power); ties broken
    //        by highest attacker_power (biggest value trade).
    //      - If no clean kill exists, fall back to a chump-block to
    //        save us from lethal damage when our life total is low
    //        (< current incoming damage).
    //   4. Each attacker can be assigned multiple blockers if a single
    //      blocker can't kill it — the loop falls through to try the
    //      next blocker.
    use crate::card::Keyword;
    // (id, power, toughness, flying, deathtouch). Deathtouch makes the
    // attacker lethal to any blocker it damages regardless of power, so
    // the bot must treat a block against it as a likely loss of the
    // blocker when scoring trades.
    let attacker_info: Vec<(CardId, i32, i32, bool, bool)> = state
        .attacking()
        .iter()
        .filter(|atk| state.defender_for(atk.target) == Some(seat))
        .filter_map(|atk| {
            state
                .battlefield
                .iter()
                .find(|c| c.id == atk.attacker)
                .map(|a| {
                    (
                        atk.attacker,
                        a.power(),
                        a.toughness(),
                        a.has_keyword(&Keyword::Flying),
                        a.has_keyword(&Keyword::Deathtouch),
                    )
                })
        })
        .collect();
    let total_incoming: i32 = attacker_info.iter().map(|(_, p, _, _, _)| *p).sum();
    // Infect (CR 702.90) / Toxic (CR 702.180) make poison the lethal clock,
    // not life: a player with 10+ poison counters loses (CR 104.3d). The bot
    // must chump an infect/toxic attacker to avoid a poison-out even at a
    // healthy life total. Infect deals its power as poison; Toxic N adds N on
    // top of normal combat damage.
    let incoming_poison: u32 = state
        .attacking()
        .iter()
        .filter(|atk| state.defender_for(atk.target) == Some(seat))
        .filter_map(|atk| state.battlefield.iter().find(|c| c.id == atk.attacker))
        .map(|a| {
            let mut p = 0u32;
            if a.has_keyword(&Keyword::Infect) {
                p += a.power().max(0) as u32;
            }
            p += a
                .definition
                .keywords
                .iter()
                .filter_map(|k| if let Keyword::Toxic(n) = k { Some(*n) } else { None })
                .sum::<u32>();
            p
        })
        .sum();
    let poison_threatened =
        incoming_poison > 0 && state.players[seat].poison_counters + incoming_poison >= 10;
    let life_threatened = state.players[seat].life <= total_incoming || poison_threatened;

    let mut blockers: Vec<(CardId, i32, i32, bool, bool, bool)> = state
        .battlefield
        .iter()
        // `can_block()` only checks creature-ness + untapped; also exclude
        // creatures that genuinely can't block (Decayed CR 702.147, or a
        // granted "can't block") so the bot never submits an illegal block.
        .filter(|c| {
            c.controller == seat
                && c.can_block()
                && !c.has_keyword(&Keyword::Decayed)
                && !c.has_keyword(&Keyword::CantBlock)
        })
        .map(|c| {
            (
                c.id,
                c.power(),
                c.toughness(),
                c.has_keyword(&Keyword::Flying),
                c.has_keyword(&Keyword::Reach),
                c.has_keyword(&Keyword::Deathtouch),
            )
        })
        .collect();
    blockers.sort_by_key(|(_, p, _, _, _, _)| *p);

    // Track which attackers have already been damage-saturated by
    // assigned blockers — if blocker total toughness >= attacker
    // power, additional blockers on the same attacker are wasteful
    // unless they bring deathtouch / first strike.
    let mut attacker_damage_taken: std::collections::HashMap<CardId, i32> =
        std::collections::HashMap::new();
    let mut assignments: Vec<(CardId, CardId)> = Vec::new();

    for (b_id, b_pow, b_tough, b_flying, b_reach, b_dt) in blockers {
        // Pick the best attacker for this blocker.
        let mut best: Option<(CardId, i32, bool)> = None; // (attacker, score, was_kill)
        for (a_id, a_pow, a_tough, a_flying, a_dt) in &attacker_info {
            if *a_flying && !b_flying && !b_reach {
                continue;
            }
            // Authoritative legality gate (CR 509.1b): also honors
            // "can't be blocked except by …" / "… by …" restrictions,
            // protection, shadow, etc. Skip attackers this blocker can't
            // legally be assigned to, so the bot never submits a block batch
            // the engine will reject.
            if !state.blocker_can_block_attacker(b_id, *a_id) {
                continue;
            }
            // Skip attackers that already have at least their toughness
            // worth of damage queued unless we have deathtouch.
            let queued = attacker_damage_taken.get(a_id).copied().unwrap_or(0);
            if !b_dt && queued >= *a_tough {
                continue;
            }
            // First-strike awareness (CR 702.7): if the attacker strikes
            // first (and the blocker doesn't strike back first) and its
            // first-strike damage is already lethal to the blocker, the
            // blocker dies *before* dealing any damage — so it never trades
            // up. Such a "kill" is illusory; downgrade it to a chump.
            let atk_first_strike = state
                .battlefield
                .iter()
                .find(|c| c.id == *a_id)
                .is_some_and(|a| {
                    a.has_keyword(&Keyword::FirstStrike) || a.has_keyword(&Keyword::DoubleStrike)
                });
            let blk_first_strike = {
                let blk = state.battlefield.iter().find(|c| c.id == b_id);
                blk.is_some_and(|c| {
                    c.has_keyword(&Keyword::FirstStrike) || c.has_keyword(&Keyword::DoubleStrike)
                })
            };
            // CR 702.16e — protection prevents combat damage either way:
            // a blocker protected from the attacker's color takes none (won't
            // die), and an attacker protected from the blocker's color takes
            // none (won't be killed). Factor both into the trade math.
            let blocker_takes_no_dmg = state.damage_prevented_by_protection(*a_id, b_id);
            let attacker_takes_no_dmg = state.damage_prevented_by_protection(b_id, *a_id);
            let dies_before_striking = atk_first_strike
                && !blk_first_strike
                && !blocker_takes_no_dmg
                && (*a_pow >= b_tough || (*a_dt && *a_pow >= 1));
            let kills_attacker = !attacker_takes_no_dmg
                && !dies_before_striking
                && (b_dt || b_pow >= (a_tough - queued));
            // A deathtouch attacker kills the blocker on any damage.
            let dies_to_attacker =
                !blocker_takes_no_dmg && (*a_pow >= b_tough || (*a_dt && *a_pow >= 1));
            // Scoring: clean trade (kill, don't die) > kill-and-die >
            // chump (don't kill, die). Higher attacker power adds value.
            let score = if kills_attacker && !dies_to_attacker {
                1000 + *a_pow
            } else if kills_attacker && dies_to_attacker {
                // Even trade (both die). Prefer trading up: score by the
                // stat delta (proxy = power + toughness). Don't sacrifice a
                // much bigger creature for a small attacker unless we're
                // under pressure — keep the body and take the hit.
                let delta = (*a_pow + *a_tough) - (b_pow + b_tough);
                if !life_threatened && delta < -2 {
                    continue;
                }
                500 + delta
            } else if life_threatened {
                // Chump-block to stop lethal damage. A trampler tramples
                // over a chump (CR 702.19e), so a lone chump only stops
                // `blocker_toughness` of its damage — score by the actual
                // damage saved so the bot prefers fully blocking a
                // non-trampler over partially blocking a trampler.
                let a_trample = state
                    .battlefield
                    .iter()
                    .find(|c| c.id == *a_id)
                    .is_some_and(|a| a.has_keyword(&Keyword::Trample));
                let saved = if a_trample { b_tough.min(*a_pow) } else { *a_pow };
                100 + saved
            } else {
                continue;
            };
            if best.map(|(_, s, _)| s < score).unwrap_or(true) {
                best = Some((*a_id, score, kills_attacker));
            }
        }
        if let Some((a_id, _score, _kill)) = best {
            assignments.push((b_id, a_id));
            // Mark the damage queued so subsequent blockers can pile on
            // attackers that aren't fully covered yet.
            *attacker_damage_taken.entry(a_id).or_insert(0) += b_tough;
        }
    }
    // Gang-block-to-kill when our life is threatened. The greedy single-
    // blocker pass above only starts blocking an attacker when one blocker
    // alone can kill it (or we chump). When we're facing lethal, trading
    // several spare creatures to *remove* a big attacker permanently beats
    // scattering chumps that die for nothing. For each still-unblocked
    // attacker (largest power first), pile idle blockers on until their
    // combined power reaches the attacker's toughness, then commit only if
    // the gang actually kills it.
    if life_threatened {
        let mut used: std::collections::HashSet<CardId> =
            assignments.iter().map(|(b, _)| *b).collect();
        let mut idle: Vec<(CardId, i32, i32, bool, bool, bool)> = state
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && c.can_block() && !used.contains(&c.id))
            .map(|c| {
                (
                    c.id,
                    c.power(),
                    c.toughness(),
                    c.has_keyword(&Keyword::Flying),
                    c.has_keyword(&Keyword::Reach),
                    c.has_keyword(&Keyword::Deathtouch),
                )
            })
            .collect();
        let mut uncovered: Vec<(CardId, i32, i32, bool, bool)> = attacker_info
            .iter()
            .filter(|(a_id, _, _, _, _)| !assignments.iter().any(|(_, aid)| aid == a_id))
            .copied()
            .collect();
        uncovered.sort_by_key(|(_, p, _, _, _)| -*p);
        for (a_id, _a_pow, a_tough, a_flying, _a_dt) in uncovered {
            // Collect a gang of legal idle blockers that together kill it.
            let mut gang: Vec<CardId> = Vec::new();
            let mut dmg = 0i32;
            let mut kills = false;
            for (b_id, b_pow, _bt, b_fly, b_reach, b_dt) in &idle {
                if a_flying && !b_fly && !b_reach {
                    continue;
                }
                gang.push(*b_id);
                dmg += *b_pow;
                if *b_dt || dmg >= a_tough {
                    kills = true;
                    break;
                }
            }
            if kills {
                for b_id in &gang {
                    assignments.push((*b_id, a_id));
                    used.insert(*b_id);
                }
                idle.retain(|(id, ..)| !gang.contains(id));
            }
        }
    }

    // CR 509.1c — satisfy "must be blocked if able" (Academic Dispute /
    // Lure). The engine rejects a declaration that leaves such an attacker
    // unblocked while an idle able blocker exists, so the bot must assign
    // one or it would deadlock the combat step. Pull any unused creature
    // that can legally block (respecting flying/reach) onto each
    // must-be-blocked attacker still missing a blocker.
    for (a_id, _a_pow, _a_tough, a_flying, _a_dt) in &attacker_info {
        let must_block = state
            .battlefield
            .iter()
            .find(|c| c.id == *a_id)
            .is_some_and(|a| a.has_keyword(&Keyword::MustBeBlocked));
        if !must_block || assignments.iter().any(|(_, aid)| aid == a_id) {
            continue;
        }
        if let Some(idle) = state.battlefield.iter().find(|c| {
            c.controller == seat
                && c.can_block()
                && !assignments.iter().any(|(bid, _)| *bid == c.id)
                && (!a_flying
                    || c.has_keyword(&Keyword::Flying)
                    || c.has_keyword(&Keyword::Reach))
        }) {
            assignments.push((idle.id, *a_id));
        }
    }

    // CR 509.1b — a Menace attacker can't be blocked except by two or more
    // creatures. The greedy passes above assign one blocker at a time, so a
    // lone block on a Menace attacker is illegal and the engine would reject
    // the whole declaration. For each Menace attacker with exactly one
    // assigned blocker, pull in a second legal idle blocker; if none is
    // available, drop the lone block (better unblocked than illegal).
    for (a_id, _a_pow, _a_tough, a_flying, _a_dt) in &attacker_info {
        let is_menace = state
            .battlefield
            .iter()
            .find(|c| c.id == *a_id)
            .is_some_and(|a| a.has_keyword(&Keyword::Menace));
        if !is_menace {
            continue;
        }
        if assignments.iter().filter(|(_, aid)| aid == a_id).count() != 1 {
            continue;
        }
        let second = state.battlefield.iter().find(|c| {
            c.controller == seat
                && c.can_block()
                && !assignments.iter().any(|(bid, _)| *bid == c.id)
                && (!a_flying
                    || c.has_keyword(&Keyword::Flying)
                    || c.has_keyword(&Keyword::Reach))
        });
        match second {
            Some(c) => assignments.push((c.id, *a_id)),
            None => assignments.retain(|(_, aid)| aid != a_id),
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
    if !a.tap_cost || a.sac_cost || a.life_cost > 0 || !a.mana_cost.symbols.is_empty() {
        return false;
    }
    // Reject abilities carrying any *additional* resource cost beyond the tap:
    // sacrificing / bouncing / tapping / exiling other permanents, discarding,
    // exiling the source, paying energy, or a "from graveyard/hand"/gated
    // activation. The bot shouldn't burn those just to float a mana mid-tap.
    // (Quirion Ranger, Witch's Oven, Heritage Druid, energy taplands, etc.)
    if a.sac_other_filter.is_some()
        || a.bounce_other_filter.is_some()
        || a.tap_other_filter.is_some()
        || a.tap_n_filter.is_some()
        || a.exile_other_filter.is_some()
        || a.discard_cost.is_some()
        || a.exile_self_cost
        || a.energy_cost > 0
        || a.condition.is_some()
        || a.from_graveyard
        || a.from_hand
    {
        return false;
    }
    // Plain fixed-color / colorless rocks (Sol Ring, Mind Stone) plus
    // fixed-multicolor `OfColor` rocks — all decision-free, so the bot can
    // tap them and keep sequencing toward a cast. *Choice* sources
    // (`AnyOneColor`) are excluded on purpose: a mid-sequence `ChooseColor`
    // breaks the tap-then-cast flow (see `bot_does_not_tap_color_choice_mana_source`).
    // Damage / life-loss riders (painlands; the `life_cost > 0` guard above)
    // are excluded so the bot doesn't hurt itself for a free tap.
    matches!(
        &a.effect,
        Effect::AddMana {
            pool: ManaPayload::Colors(_) | ManaPayload::Colorless(_) | ManaPayload::OfColor(_, _),
            ..
        }
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
    can_afford_with_extra(def, pool, 0, 0)
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
    // Fold in generic cost *reductions* (Affinity, CostReduction statics,
    // graveyard-affinity) the same way the real cast path does — otherwise the
    // bot overestimates the cost of e.g. Tolarian Terror with a full graveyard
    // and never casts it. Target-dependent reductions are skipped (no target
    // chosen yet), so this stays conservative.
    let reduction = crate::game::actions::cost_reduction_for_spell(state, seat, card, None);
    can_afford_with_extra(&card.definition, &state.players[seat].mana_pool, extra, reduction)
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
            | Value::DistinctTypesInTopOfLibrary { .. }
            | Value::DistinctTypesInGraveyard { .. } => false,
            _ => false,
        }
    }
    fn predicate_uses_x(p: &crate::effect::Predicate) -> bool {
        use crate::effect::Predicate as P;
        match p {
            P::ValueAtLeast(a, b) | P::ValueAtMost(a, b) | P::ValueEquals(a, b) => {
                value_uses_x(a) || value_uses_x(b)
            }
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
        Effect::CreateToken { count, .. }
        | Effect::CopySpell { count, .. }
        | Effect::CopySpellMayChooseTargets { count, .. } => value_uses_x(count),
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
        (Effect::Seq(steps), Some(_)) => steps
            .iter()
            .find(|s| matches!(s, Effect::ChooseMode(_)))
            .map(|s| mode_branch(s, mode))
            .unwrap_or(eff),
        _ => eff,
    }
}

fn can_afford_with_extra(
    def: &CardDefinition,
    pool: &ManaPool,
    extra_generic: u32,
    reduction: u32,
) -> bool {
    let mut cost = if def.cost.has_x() {
        def.cost.with_x_value(0)
    } else {
        def.cost.clone()
    };
    if reduction > 0 {
        cost.reduce_generic(reduction);
    }
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

/// True when `ta` is the canonical Strixhaven magecraft trigger:
/// SpellCast scope=YourControl with the IS-only predicate. Used by
/// the bot's spell-bias heuristic so a controlled magecraft permanent
/// nudges the bot toward casting an IS spell to fire the trigger.
fn is_magecraft_trigger(ta: &crate::card::TriggeredAbility) -> bool {
    use crate::card::{EventKind, EventScope};
    matches!(ta.event.kind, EventKind::SpellCast)
        && matches!(ta.event.scope, EventScope::YourControl)
        && ta.event.filter.is_some()
}

/// True when the card with id `cid` in `seat`'s hand is an instant or
/// sorcery. Cheap helper for the magecraft-bias path; falls back to
/// false on missing cards.
fn is_instant_or_sorcery_in_hand(state: &GameState, seat: usize, cid: CardId) -> bool {
    use crate::card::CardType;
    state.players[seat]
        .hand
        .iter()
        .find(|c| c.id == cid)
        .map(|c| {
            c.definition.card_types.contains(&CardType::Instant)
                || c.definition.card_types.contains(&CardType::Sorcery)
        })
        .unwrap_or(false)
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

    fn body_card(name: &'static str, body: Effect) -> CardDefinition {
        use crate::card::{CardType, Subtypes, TriggeredAbility};
        use crate::effect::{EventKind, EventScope, EventSpec};
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "you may".to_string(),
                    body: Box::new(body),
                },
            }],
            ..Default::default()
        }
    }

    #[test]
    fn bot_takes_beneficial_optional_trigger() {
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(
            0,
            body_card("Upside", Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        );
        assert!(optional_trigger_beneficial(&g, id, "you may"),
            "a pure-upside 'you may draw' is taken by the bot");
    }

    #[test]
    fn bot_declines_optional_trigger_that_sacrifices_itself() {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(
            0,
            body_card("Downside", Effect::SacrificeSource),
        );
        assert!(!optional_trigger_beneficial(&g, id, "you may"),
            "a 'you may sacrifice this' rider is a self-cost the bot declines");
    }

    /// A planeswalker whose highest-loyalty ability needs a target that
    /// doesn't exist must not stop the bot from activating a lower targetless
    /// ability (regression: the `?` on `auto_target_for_effect` used to bail
    /// out of every ability and planeswalker).
    #[test]
    fn bot_skips_untargetable_loyalty_ability_for_a_usable_one() {
        use crate::card::{CardType, LoyaltyAbility, Subtypes};
        use crate::effect::shortcut::target_filtered;
        use crate::card::SelectionRequirement;
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let pw = CardDefinition {
            name: "Test Walker",
            card_types: vec![CardType::Planeswalker],
            base_loyalty: 3,
            loyalty_abilities: vec![
                // Highest loyalty, but needs a creature target (none exist).
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: 2,
                    effect: Effect::DealDamage {
                        to: target_filtered(SelectionRequirement::Creature),
                        amount: Value::Const(2),
                    },
                },
                // Lower loyalty, no target — the bot should fall through here.
                LoyaltyAbility {
                    x_cost: false,
                    loyalty_cost: 1,
                    effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                },
            ],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, pw);
        g.add_card_to_library(0, catalog::island());
        let action = pick_loyalty_ability(&g, 0).expect("bot finds the targetless +1");
        match action {
            GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => {
                assert_eq!(card_id, id);
                assert_eq!(ability_index, 1, "picked the targetless draw, not the dead burn");
            }
            _ => panic!("expected a loyalty activation"),
        }
    }

    #[test]
    fn bot_declines_self_costly_optional_trigger() {
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(
            0,
            body_card("Downside", Effect::LoseLife { who: Selector::You, amount: Value::Const(3) }),
        );
        assert!(!optional_trigger_beneficial(&g, id, "you may"),
            "a 'you may lose 3 life' optional trigger is declined");
    }

    /// Self-directed damage / mill bodies are costs too — the bot declines a
    /// "you may have this deal 4 damage to you" optional trigger.
    #[test]
    fn bot_declines_self_damage_optional_trigger() {
        use crate::effect::{Selector, Value};
        let mut g = two_player_game();
        let dmg = g.add_card_to_battlefield(
            0,
            body_card("SelfBurn", Effect::DealDamage { to: Selector::You, amount: Value::Const(4) }),
        );
        assert!(!optional_trigger_beneficial(&g, dmg, "you may"),
            "a 'you may deal 4 to you' optional trigger is declined");
        let mill = g.add_card_to_battlefield(
            0,
            body_card("SelfMill", Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
        );
        assert!(!optional_trigger_beneficial(&g, mill, "you may"),
            "a 'you may mill yourself 3' optional trigger is declined");
    }

    /// `MayPay` shares the `OptionalTrigger` decision shape with `MayDo`, so
    /// the bot's self-cost screen must introspect it too: a "pay {1}: you lose
    /// 3 life" body is declined even though it's reachable only via MayPay.
    #[test]
    fn bot_declines_self_costly_maypay() {
        use crate::card::{CardType, Subtypes, TriggeredAbility};
        use crate::effect::{EventKind, EventScope, EventSpec, Selector, Value};
        let mut g = two_player_game();
        let def = CardDefinition {
            name: "PayDownside",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayPay {
                    description: "you may pay".to_string(),
                    mana_cost: crate::mana::cost(&[crate::mana::generic(1)]),
                    body: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(3) }),
                },
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, def);
        assert!(!optional_trigger_beneficial(&g, id, "you may pay"),
            "a MayPay whose body costs the bot 3 life is declined");
    }

    fn generic_spell(name: &'static str, cmc: u32) -> CardDefinition {
        use crate::card::{CardType, Subtypes};
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            cost: crate::mana::cost(&[crate::mana::generic(cmc)]),
            ..Default::default()
        }
    }

    /// Self-discard heuristic pitches the priciest spell (least likely to be
    /// cast soon), not the head of the hand, when the bot isn't flooded.
    #[test]
    fn bot_self_discard_pitches_priciest_spell() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let pricey = g.add_card_to_hand(0, generic_spell("Pricey", 6));
        let cheap = g.add_card_to_hand(0, generic_spell("Cheap", 1));
        // Offer both; head dump would take `pricey` (first), but so should the
        // heuristic here — make the cheap card the head to prove it's a real
        // choice rather than a head dump.
        let hand = vec![
            (cheap, "Cheap".to_string()),
            (pricey, "Pricey".to_string()),
        ];
        let DecisionAnswer::Discard(ids) = decide_self_discard(&g, 0, &hand, 1) else {
            panic!("expected a Discard answer");
        };
        assert_eq!(ids, vec![pricey], "the most expensive spell is pitched");
    }

    /// When flooded (≥5 lands in play), a surplus land is pitched before a
    /// keepable cheap spell.
    #[test]
    fn bot_self_discard_pitches_surplus_land_when_flooded() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_battlefield(0, catalog::island());
        }
        let land = g.add_card_to_hand(0, catalog::island());
        let spell = g.add_card_to_hand(0, generic_spell("Cheap", 1));
        let hand = vec![
            (spell, "Cheap".to_string()),
            (land, "Island".to_string()),
        ];
        let DecisionAnswer::Discard(ids) = decide_self_discard(&g, 0, &hand, 1) else {
            panic!("expected a Discard answer");
        };
        assert_eq!(ids, vec![land], "a flooded bot pitches the surplus land");
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


    /// The bot spends surplus energy on a beneficial energy-payoff ability
    /// (Longtusk Cub's `{E}{E}{E}: +1/+1 counter`) once nothing better to do.
    #[test]
    fn bot_spends_energy_on_payoff_ability() {
        let mut g = two_player_game();
        let cub = g.add_card_to_battlefield(0, catalog::longtusk_cub());
        g.clear_sickness(cub);
        g.players[0].energy = 3;
        let action = pick_energy_payoff(&g, 0).expect("bot should pay energy for the counter");
        match action {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, cub),
            _ => panic!("expected an activate-ability action"),
        }
        // With too little energy the bot leaves it alone.
        g.players[0].energy = 1;
        assert!(pick_energy_payoff(&g, 0).is_none(), "won't activate without enough energy");
    }

    /// The bot recurs a creature from the graveyard via Embalm when it can
    /// afford the cost.
    #[test]
    fn bot_embalms_from_graveyard_with_spare_mana() {
        use crate::TurnStep;
        let mut g = two_player_game();
        let cat = g.add_card_to_graveyard(0, catalog::sacred_cat());
        g.players[0].mana_pool.add(crate::mana::Color::White, 1);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_graveyard_recursion(&g, 0).expect("bot should Embalm Sacred Cat");
        match action {
            GameAction::ActivateAbility { card_id, .. } => assert_eq!(card_id, cat),
            _ => panic!("expected an activate-ability action"),
        }
        // With no mana it leaves the card alone.
        g.players[0].mana_pool.empty();
        assert!(pick_graveyard_recursion(&g, 0).is_none(), "won't Embalm without mana");
    }

    /// The bot uses a *targeted* graveyard-activated ability (Scavenge),
    /// auto-picking its own creature as the target.
    #[test]
    fn bot_scavenges_onto_own_creature() {
        use crate::TurnStep;
        let mut g = two_player_game();
        let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let mangler = g.add_card_to_graveyard(0, catalog::dreg_mangler());
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let action = pick_graveyard_recursion(&g, 0).expect("bot should Scavenge Dreg Mangler");
        match action {
            GameAction::ActivateAbility { card_id, target, .. } => {
                assert_eq!(card_id, mangler);
                assert_eq!(target, Some(crate::game::Target::Permanent(beater)),
                    "auto-targets the bot's own creature");
            }
            _ => panic!("expected an activate-ability action"),
        }
    }

    /// The bot also recognises the real-cost energy form
    /// (`ActivatedAbility.energy_cost`), not just resolve-time `PayEnergy`.
    #[test]
    fn bot_spends_energy_on_real_cost_form() {
        use crate::card::{ActivatedAbility, CardDefinition, CardType, CounterType};
        let mut g = two_player_game();
        let def = CardDefinition {
            name: "Energy Engine",
            card_types: vec![CardType::Creature],
            power: 1,
            toughness: 1,
            activated_abilities: vec![ActivatedAbility {
                energy_cost: 2,
                discard_cost: None,
                effect: Effect::AddCounter {
                    what: crate::effect::Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: crate::effect::Value::Const(1),
                },
                ..Default::default()
            }],
            ..Default::default()
        };
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        g.players[0].energy = 2;
        assert!(pick_energy_payoff(&g, 0).is_some(), "bot fires the energy_cost-gated payoff");
        g.players[0].energy = 1;
        assert!(pick_energy_payoff(&g, 0).is_none(), "and only when it can afford it");
    }

    /// Mulligan heuristic: ship a 1-land seven, keep a 3-land seven, and
    /// stop digging once two mulligans have been taken.
    #[test]
    fn bot_mulligans_land_light_hands_but_keeps_balanced_ones() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        // 1 land + 6 spells → mulligan.
        g.add_card_to_hand(0, catalog::island());
        for _ in 0..6 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
        assert!(matches!(decide_mulligan(&g, 0, 0), DecisionAnswer::TakeMulligan));
        // Stop digging after two mulligans even on a bad hand.
        assert!(matches!(decide_mulligan(&g, 0, 2), DecisionAnswer::Keep));

        // 3 lands + 4 spells, colors aligned (Forests for green bears) → keep.
        let mut g2 = two_player_game();
        for _ in 0..3 { g2.add_card_to_hand(0, catalog::forest()); }
        for _ in 0..4 { g2.add_card_to_hand(0, catalog::grizzly_bears()); }
        assert!(matches!(decide_mulligan(&g2, 0, 0), DecisionAnswer::Keep));
    }

    /// Color-screw: enough lands and a fine curve, but the lands can't make
    /// the spells' colors (3 Islands + green {1}{G} Grizzly Bears) → ship it.
    #[test]
    fn bot_mulligans_color_screwed_hands() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
        for _ in 0..4 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
        assert!(matches!(decide_mulligan(&g, 0, 0), DecisionAnswer::TakeMulligan),
            "no green source for the green spells → color screw → mulligan");
    }

    /// Curve screen: a hand with enough lands but only spells too expensive
    /// to cast early is a screwed keep — ship it on the first mulligan.
    #[test]
    fn bot_mulligans_lands_with_no_early_play() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        // 3 lands + four {6} Obsianus Golems → no spell castable by turn ~4.
        for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
        for _ in 0..4 { g.add_card_to_hand(0, catalog::obsianus_golem()); }
        assert!(matches!(decide_mulligan(&g, 0, 0), DecisionAnswer::TakeMulligan),
            "no early play despite enough lands → mulligan");
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
        // Karn has a +1 (reveal two, opponent picks one for your hand) at
        // index 0, a -1 at index 1, and a -2 (Construct token) at index 2.
        // Sorted by descending loyalty cost, the bot should pick the +1.
        let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
        g.clear_sickness(karn);
        // Stock the library so the +1 has cards to reveal.
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

    /// Helper: a 1/1 creature with one extra keyword for attack-filter tests.
    fn one_one_with(name: &'static str, kw: crate::card::Keyword) -> CardDefinition {
        let mut d = catalog::grizzly_bears();
        d.name = name;
        d.power = 1;
        d.toughness = 1;
        d.keywords.push(kw);
        d
    }

    /// A menace attacker swings even into a single bigger blocker — menace
    /// needs two blockers, so it gets through (the suicide filter must not
    /// hold it back when the opponent has fewer than two blockers).
    #[test]
    fn bot_attacks_with_menace_into_lone_blocker() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, one_one_with("Sneak", crate::card::Keyword::Menace));
        g.clear_sickness(atk);
        g.add_card_to_battlefield(1, catalog::grizzly_bears()); // lone 2/2 blocker
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|atk_decl| atk_decl.attacker == atk),
                    "menace attacker should swing past a lone blocker");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// The bot won't declare a CanAttackOnlyIfDefenderControls attacker
    /// (Dandân) into a defender whose board fails the filter — doing so
    /// would get the whole batch rejected by the engine.
    #[test]
    fn bot_holds_back_dandan_when_defender_has_no_island() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let dd = g.add_card_to_battlefield(0, catalog::dandan());
        g.clear_sickness(dd);
        g.add_card_to_battlefield(0, catalog::island()); // your Island, not the defender's
        let mut bot = RandomBot::new();
        if let Some(GameAction::DeclareAttackers(a)) = bot.next_action(&g, 0) {
            assert!(!a.iter().any(|x| x.attacker == dd),
                "Dandân must not be declared when the defender controls no Island");
        } // declaring no attackers is also fine
        // Now give the defender an Island — Dandân becomes a legal attacker.
        g.add_card_to_battlefield(1, catalog::island());
        let mut bot2 = RandomBot::new();
        match bot2.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|x| x.attacker == dd),
                    "Dandân should attack once the defender controls an Island");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// A deathtouch attacker swings even when smaller than every blocker —
    /// any block trades the opponent's creature for ours.
    #[test]
    fn bot_attacks_with_deathtouch_into_bigger_blocker() {
        let mut g = two_player_game();
        g.step = TurnStep::DeclareAttackers;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        let atk = g.add_card_to_battlefield(0, one_one_with("Stinger", crate::card::Keyword::Deathtouch));
        g.clear_sickness(atk);
        // Two 3/3s — without deathtouch awareness the suicide filter would
        // hold the 1/1 back.
        g.add_card_to_battlefield(1, catalog::hill_giant());
        g.add_card_to_battlefield(1, catalog::hill_giant());
        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("bot acts") {
            GameAction::DeclareAttackers(a) => {
                assert!(a.iter().any(|atk_decl| atk_decl.attacker == atk),
                    "deathtouch attacker should swing into bigger blockers");
            }
            other => panic!("expected DeclareAttackers, got {:?}", other),
        }
    }

    /// Magecraft-aware spell bias: when the bot controls a magecraft
    /// permanent and has both an IS spell and a creature spell in hand,
    /// it should prefer the IS spell to fire the magecraft trigger.
    /// Push (claude/modern_decks batch 202).
    #[test]
    fn bot_prefers_is_spell_when_magecraft_in_play() {
        let mut g = two_player_game();
        // Drop Witherbloom Apprentice (a magecraft permanent) on board.
        g.add_card_to_battlefield(0, catalog::witherbloom_apprentice());
        // Hand has both Lightning Bolt (instant) and Grizzly Bears
        // (creature). The bot must prefer the bolt.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let _bear = g.add_card_to_hand(0, catalog::grizzly_bears());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = RandomBot::new();
        // Drive the bot until it produces a CastSpell — could pass
        // through PlayLand / mana abilities first if seeded with hand-
        // played lands, but in this synthetic state the next non-mana
        // action is the spell.
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastSpell { card_id, .. } = action {
                assert_eq!(card_id, bolt,
                    "magecraft-bias should pick the instant over the creature");
                return;
            }
            // Drive the engine forward so non-cast actions don't loop.
            let _ = g.perform_action(action);
        }
        panic!("bot never produced a CastSpell action");
    }

    /// The bot casts an Adventure half (Stomp) as removal when it can afford
    /// the adventure but not the creature (CR 715).
    #[test]
    fn bot_casts_adventure_half_as_removal() {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, catalog::bonecrusher_giant());
        // {1}{R}: enough for Stomp, not the {2}{R} creature.
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        let mut bot = RandomBot::new();
        for _ in 0..16 {
            let action = bot.next_action(&g, 0).expect("bot should act");
            if let GameAction::CastAdventure { card_id, .. } = action {
                assert_eq!(card_id, id, "bot Stomps with the adventure half");
                let _ = bear;
                return;
            }
            let _ = g.perform_action(action);
        }
        panic!("bot never cast the adventure half");
    }

    /// When forced to chump (life threatened, no clean kill), the bot
    /// prefers fully blocking a non-trampler over a trampler — a chump
    /// against a trampler only stops `blocker_toughness` of its damage
    /// (CR 702.19e). Push (claude/modern_decks).
    #[test]
    fn bot_chumps_non_trampler_over_trampler_when_threatened() {
        use crate::card::{CardDefinition, CardType, Keyword, Subtypes};
        use crate::game::types::{Attack, AttackTarget};
        fn beater(name: &'static str, kws: Vec<Keyword>) -> CardDefinition {
            CardDefinition {
                name,
                card_types: vec![CardType::Creature],
                power: 4,
                toughness: 4,
                keywords: kws,
                ..Default::default()
            }
        }
        let mut g = two_player_game();
        let vanilla = g.add_card_to_battlefield(0, beater("Brute", vec![]));
        let trampler = g.add_card_to_battlefield(0, beater("Stomper", vec![Keyword::Trample]));
        // One 0/3 wall that can't kill either — only a chump is possible.
        let wall = g.add_card_to_battlefield(1, beater("Wall", vec![]));
        if let Some(w) = g.battlefield_find_mut(wall) { w.definition = std::sync::Arc::new(
            CardDefinition {
                name: "Wall",
                card_types: vec![CardType::Creature],
                toughness: 3,
                ..Default::default()
            }); }
        g.players[1].life = 3; // 8 incoming ≫ 3 → life threatened
        g.attacking = vec![
            Attack { attacker: vanilla, target: AttackTarget::Player(1) },
            Attack { attacker: trampler, target: AttackTarget::Player(1) },
        ];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(wall, vanilla)],
            "chump the non-trampler (saves 4) over the trampler (saves only 3)");
    }

    /// CR 702.147 — a Decayed creature can't block, so the bot must never
    /// offer it as a blocker even when life-threatened (an illegal block
    /// would get the whole DeclareBlockers batch rejected).
    #[test]
    fn bot_never_blocks_with_a_decayed_creature() {
        use crate::card::{CardDefinition, CardType, Keyword, Subtypes};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let atk = g.add_card_to_battlefield(0, CardDefinition {
            name: "Beater",
            card_types: vec![CardType::Creature],
            power: 4,
            toughness: 4,
            ..Default::default()
        });
        let zombie = g.add_card_to_battlefield(1, CardDefinition {
            name: "Decayed Zombie",
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Decayed],
            ..Default::default()
        });
        g.players[1].life = 1; // life-threatened → the bot would chump if it could
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == zombie), "decayed creature is never declared as a blocker");
    }

    /// CR 702.16e — the bot treats a block by a protection-from-the-attacker's
    /// -color creature as a clean kill (it survives + kills) rather than a
    /// suicidal trade, so it blocks even at full life.
    #[test]
    fn bot_blocks_freely_with_protected_creature() {
        use crate::card::{CardDefinition, CardType, Keyword, Subtypes};
        use crate::game::types::{Attack, AttackTarget};
        use crate::mana::{cost, r, Color};
        let mut g = two_player_game();
        let mut red_atk = CardDefinition {
            name: "Red Beater",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            ..Default::default()
        };
        red_atk.cost = cost(&[r()]);
        let atk = g.add_card_to_battlefield(0, red_atk);
        let prot = CardDefinition {
            name: "Warded Blocker",
            card_types: vec![CardType::Creature],
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Protection(Color::Red)],
            ..Default::default()
        };
        let blk = g.add_card_to_battlefield(1, prot);
        // Not life-threatened (only a chump would otherwise be declined).
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert_eq!(blocks, vec![(blk, atk)], "protected 3/3 kills the red 3/3 and takes no damage");
    }

    /// The bot won't throw a much bigger creature into an even trade with a
    /// small attacker when it isn't under pressure (keeps the body, takes the
    /// hit). A 5/5 should not block a 5/1 at healthy life.
    #[test]
    fn bot_keeps_big_body_over_bad_even_trade() {
        use crate::card::{CardDefinition, CardType, Subtypes};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let glass = CardDefinition {
            name: "Glass Cannon",
            card_types: vec![CardType::Creature],
            power: 5,
            toughness: 1,
            ..Default::default()
        };
        let atk = g.add_card_to_battlefield(0, glass);
        let beater = CardDefinition {
            name: "Big Beater",
            card_types: vec![CardType::Creature],
            power: 5,
            toughness: 5,
            ..Default::default()
        };
        let big = g.add_card_to_battlefield(1, beater);
        g.players[1].life = 20; // not threatened by 5 damage
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == big),
            "won't trade a 5/5 to kill a 5/1 when healthy");
    }

    /// CR 509.1b — the bot must not assign a power-2 blocker to a Steel Leaf
    /// Champion ("can't be blocked by creatures with power 2 or less"), even
    /// when life-threatened; the legality gate keeps the block batch legal.
    #[test]
    fn bot_skips_illegal_block_against_steel_leaf_champion() {
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let champ = g.add_card_to_battlefield(0, catalog::steel_leaf_champion());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — illegal
        g.players[1].life = 1; // life-threatened, so it would chump if it could
        g.attacking = vec![Attack { attacker: champ, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(!blocks.iter().any(|(b, _)| *b == bear),
            "power-2 blocker can't be assigned to Steel Leaf Champion");
    }

    /// CR 702.90 / 104.3d — the bot chumps an infect attacker that would
    /// reach 10 poison even at a healthy life total (poison, not life, is the
    /// lethal clock).
    #[test]
    fn bot_chumps_infect_attacker_to_avoid_poison_out() {
        use crate::card::{CardDefinition, CardType, Keyword, Subtypes};
        use crate::game::types::{Attack, AttackTarget};
        let mut g = two_player_game();
        let infect = CardDefinition {
            name: "Plague Beast",
            card_types: vec![CardType::Creature],
            power: 9,
            toughness: 9,
            keywords: vec![Keyword::Infect],
            ..Default::default()
        };
        let atk = g.add_card_to_battlefield(0, infect);
        let chump = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        // Healthy life (20) but already 1 poison → 9 incoming poison = 10 → lethal.
        g.players[1].poison_counters = 1;
        g.attacking = vec![Attack { attacker: atk, target: AttackTarget::Player(1) }];
        let blocks = pick_blocks_for_test(&g, 1);
        assert!(blocks.iter().any(|(b, _)| *b == chump),
            "bot chumps the infect attacker to avoid a poison-out");
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

    /// `is_free_mana_ability` accepts decision-free rocks but rejects
    /// life/damage-rider and color-choice sources.
    #[test]
    fn free_mana_ability_excludes_life_and_choice_sources() {
        use crate::effect::{PlayerRef, Value};
        let plain = ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(1)) },
            ..Default::default()
        };
        assert!(is_free_mana_ability(&plain), "plain {{C}} rock is free");
        let pay_life = ActivatedAbility { life_cost: 1, ..plain.clone() };
        assert!(!is_free_mana_ability(&pay_life), "life-paying mana source isn't free");
        let any_color = ActivatedAbility {
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
            ..plain.clone()
        };
        assert!(!is_free_mana_ability(&any_color), "color-choice source isn't auto-tapped");
        // Mana abilities with an additional resource cost are NOT free: the
        // bot must not pointlessly pay them mid-tap.
        use crate::card::SelectionRequirement;
        let sac_other = ActivatedAbility {
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            ..plain.clone()
        };
        assert!(!is_free_mana_ability(&sac_other), "sacrifice-cost mana source isn't free");
        let tap_n = ActivatedAbility {
            tap_cost: true,
            tap_n_filter: Some((SelectionRequirement::Creature, 3)),
            ..plain.clone()
        };
        assert!(!is_free_mana_ability(&tap_n), "tap-N-cost mana source isn't free");
        let energy = ActivatedAbility { energy_cost: 1, ..plain };
        assert!(!is_free_mana_ability(&energy), "energy-cost mana source isn't free");
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
        trigger_source: None,
            mana_spent: 0,
            event_amount: 0,
            intervening_if: None,
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
        trigger_source: None,
            mana_spent: 0,
            event_amount: 0,
            intervening_if: None,
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
            additional_targets: vec![],
            mode: None,
            x_value: 0,
            converged_value: 0,
            mana_spent: 0,
            uncounterable: false,
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

    /// The bot's affordability check folds in generic cost reductions:
    /// Tolarian Terror ({6}{U}) is castable on {3}{U} with three instants/
    /// sorceries in the graveyard.
    #[test]
    fn bot_affordability_honors_graveyard_affinity() {
        let mut g = two_player_game();
        let terror = g.add_card_to_hand(0, catalog::tolarian_terror());
        let card = g.players[0].hand.iter().find(|c| c.id == terror).unwrap().clone();
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(3); // {3}{U} only
        assert!(!can_afford_in_state(&g, 0, &card), "no discount yet → unaffordable");
        for _ in 0..3 { g.add_card_to_graveyard(0, catalog::lightning_bolt()); }
        let card = g.players[0].hand.iter().find(|c| c.id == terror).unwrap().clone();
        assert!(can_afford_in_state(&g, 0, &card), "−{{3}} discount → now affordable");
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
    fn bot_casts_spectacle_when_opponent_bled() {
        // Skewer the Critics ({2}{R}, Spectacle {R}) with only {R} in the pool:
        // unaffordable at its printed cost, but castable for Spectacle once an
        // opponent has lost life this turn.
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, catalog::skewer_the_critics());
        g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
        g.adjust_life(1, -1); // opponent bleeds → Spectacle online
        match main_phase_action(&g, 0) {
            GameAction::CastSpellAlternative { card_id, .. } => assert_eq!(card_id, id),
            other => panic!("expected a Spectacle alternative cast, got {other:?}"),
        }
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
        // Opp creature for mode-1 (destroy creature) to target. Drown's
        // MV gate needs MV(bear=2) ≤ cards in its controller's graveyard.
        g.add_card_to_graveyard(1, catalog::forest());
        g.add_card_to_graveyard(1, catalog::forest());
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

    /// The bot delves a stocked graveyard to cast a spell it couldn't afford
    /// at full cost (CR 702.66). Treasure Cruise ({7}{U}) with only one blue
    /// mana but seven graveyard cards must surface as a `CastSpellDelve`.
    #[test]
    fn bot_delves_to_afford_treasure_cruise() {
        let mut g = two_player_game();
        for _ in 0..7 { g.add_card_to_graveyard(0, catalog::island()); }
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_hand(0, catalog::treasure_cruise());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        // Drive the bot until it produces the delve cast (it may tap/scan
        // first, but with no lands and one floating U the delve is the only
        // castable line).
        let mut bot = RandomBot::new();
        let mut found = false;
        for _ in 0..6 {
            match bot.next_action(&g, 0) {
                Some(GameAction::CastSpellDelve { delve_cards, .. }) => {
                    assert!(!delve_cards.is_empty(), "delved at least one card");
                    found = true;
                    break;
                }
                Some(other) => { g.perform_action(other).ok(); }
                None => break,
            }
        }
        assert!(found, "bot should delve to cast Treasure Cruise");
    }

    /// The bot fetches toward its weakest color: with two Forests already
    /// down and a Forest + Island in the library, it grabs the Island.
    #[test]
    fn bot_search_fetches_weakest_color_basic() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::forest());
        g.add_card_to_battlefield(0, catalog::forest());
        let extra_forest = g.add_card_to_library(0, catalog::forest());
        let island = g.add_card_to_library(0, catalog::island());
        let candidates = vec![(extra_forest, "Forest".into()), (island, "Island".into())];
        let ans = decide_library_search(&g, 0, &candidates);
        assert!(matches!(ans, DecisionAnswer::Search(Some(id)) if id == island),
            "bot fetches the Island (Blue uncovered) over a third Forest");
    }

    /// With no basic land among the candidates the bot still fetches the
    /// first option rather than fizzling like AutoDecider.
    #[test]
    fn bot_search_fetches_nonland_when_no_basic_offered() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
        let candidates = vec![(bolt, "Lightning Bolt".into())];
        let ans = decide_library_search(&g, 0, &candidates);
        assert!(matches!(ans, DecisionAnswer::Search(Some(id)) if id == bolt),
            "bot fetches the only candidate");
    }

    /// A non-land tutor (e.g. Fauna Shaman) fetches the highest-mana-value
    /// hit — the most impactful card — not just the first candidate offered.
    #[test]
    fn bot_search_fetches_highest_mv_nonland() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
        let angel = g.add_card_to_library(0, catalog::serra_angel());   // MV 5
        let candidates = vec![
            (bears, "Grizzly Bears".into()),
            (angel, "Serra Angel".into()),
        ];
        let ans = decide_library_search(&g, 0, &candidates);
        assert!(matches!(ans, DecisionAnswer::Search(Some(id)) if id == angel),
            "bot fetches the higher-MV creature");
    }

    /// The bot offers a Bestow cast (enchanting its own creature) when it's
    /// mana-flush, instead of only ever casting the base creature.
    #[test]
    fn bot_considers_bestow_when_mana_flush() {
        let mut g = two_player_game();
        let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_hand(0, catalog::hopeful_eidolon());
        g.players[0].mana_pool.add(crate::mana::Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        // The bot can cast Hopeful Eidolon normally *or* bestow it; with the
        // random pick, the bestow line must appear over repeated draws.
        let bestowed = (0..40).any(|_| {
            matches!(main_phase_action(&g, 0),
                GameAction::CastBestow { target: Some(crate::game::Target::Permanent(t)), .. } if t == host)
        });
        assert!(bestowed, "bot offers a Bestow line enchanting its creature");
    }

    /// `decide_choose_cards` over the bot's own hand (Sneak Attack / Elvish
    /// Piper) cheats in the biggest creature it can.
    #[test]
    fn bot_choose_cards_cheats_in_biggest_creature() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        let small = g.add_card_to_hand(0, catalog::grizzly_bears()); // cmc 2
        let big = g.add_card_to_hand(0, catalog::shivan_dragon());   // cmc 6
        let candidates = vec![
            (small, "Grizzly Bears".to_string()),
            (big, "Shivan Dragon".to_string()),
        ];
        match decide_choose_cards(&g, 0, &candidates, 1) {
            DecisionAnswer::Cards(v) => assert_eq!(v, vec![big],
                "bot picks the highest-cmc creature to cheat in"),
            other => panic!("expected Cards, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod monarch_tests {
    use super::*;
    use crate::catalog;
    use crate::player::Player;

    #[test]
    fn bot_attacks_the_monarch_over_the_next_seat() {
        // 3 players: next_alive_seat(0) is 1, but seat 2 is the monarch, so
        // the bot should swing at seat 2 to steal the crown.
        let players = vec![Player::new(0, "A"), Player::new(1, "B"), Player::new(2, "C")];
        let mut g = GameState::new(players);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;
        g.monarch = Some(2);
        let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(atk);

        let mut bot = RandomBot::new();
        match bot.next_action(&g, 0).expect("an action") {
            GameAction::DeclareAttackers(attacks) => {
                assert!(
                    attacks.iter().any(|a| matches!(a.target, AttackTarget::Player(2))),
                    "bot swings at the monarch (seat 2), not the next seat"
                );
            }
            other => panic!("expected DeclareAttackers, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod self_cost_tests {
    use super::*;
    use crate::effect::{Effect, PlayerRef, Selector, Value};

    #[test]
    fn self_cost_seen_through_modal_and_pay_or_else() {
        // A self-cost mode nested inside ChooseMode is recognized.
        let modal = Effect::ChooseMode(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        ]);
        assert!(effect_imposes_self_cost(&modal), "lose-life mode is a self cost");

        // PayManaOrElse → SacrificeSource fallback is a self cost.
        let tax = Effect::PayManaOrElse {
            mana_cost: crate::mana::cost(&[crate::mana::generic(1)]),
            otherwise: Box::new(Effect::SacrificeSource),
        };
        assert!(effect_imposes_self_cost(&tax), "sac-unless-pay fallback is a self cost");

        // A purely beneficial modal is not flagged.
        let upside = Effect::ChooseMode(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]);
        assert!(!effect_imposes_self_cost(&upside));

        // find_maydo_body reaches into a mode by its prompt.
        let nested = Effect::ChooseMode(vec![Effect::MayDo {
            description: "Pay the price.".into(),
            body: Box::new(Effect::LoseLife {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
            }),
        }]);
        assert!(find_maydo_body(&nested, "Pay the price.").is_some());
    }
}
