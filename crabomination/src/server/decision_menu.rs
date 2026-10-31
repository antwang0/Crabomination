//! Every legal move at a decision: the move list a net that makes every
//! decision chooses from (phase 1 of that plan, `ML_NOTES` 2026-10-31).
//!
//! Three kinds of decision, one list shape:
//!
//! * **Priority** — [`puzzle::legal_actions`]' brute force (lands, casts
//!   and flashback casts, activated and loyalty abilities, prepared spells,
//!   each over every target), plus the bot's own cast generator, which knows
//!   the alternative casts (kicker, adventure, bestow, ...) the brute force
//!   doesn't, plus the bot's main-phase sink. Every cast and activation is
//!   then fanned out over its targets slot by slot (second targets
//!   included), its X values and its modes, each fan-out kept only if the
//!   engine accepts it (`would_accept`).
//! * **Combat** — every attack and block subset, engine-validated (the same
//!   enumeration, capped at eight creatures).
//! * **Prompts** — the pending [`Decision`]'s answers, built from its own
//!   fields (its legal targets, its cards, its count). An answer space that
//!   is a permutation or a subset is enumerated up to [`MAX_MOVES`] and the
//!   list says so when the cap binds; free-text answers (name a card, a
//!   creature type) are the decision's suggestions, also marked truncated.
//!
//! Two moves are the same move when their [`key`]s match: the action's
//! debug text with every card id replaced by the card's name (two Swamps
//! are one land drop), and the target dropped from a spell or ability that
//! takes none (the engine accepts and ignores one, so "cast X" otherwise
//! came back once per permanent).
//!
//! `bot_puzzles --menu-census` measures coverage: at every decision the
//! client's pilot makes, is its move on the list?

use crate::card::CardId;
use crate::decision::{Decision, DecisionAnswer, LearnChoice, ScryMode};
use crate::effect::Effect;
use crate::game::types::TurnStep;
use crate::game::{GameAction, GameState, Target};

use super::bot::EvalWeights;
use super::puzzle;

/// Most moves one list carries; a list cut here is marked truncated.
pub const MAX_MOVES: usize = 256;
/// Most target combinations one cast or activation fans out to.
const MAX_TARGET_COMBOS: usize = 64;
/// Highest X a fan-out tries (the engine rejects what can't be paid).
const MAX_X: u32 = 12;

/// What a decision is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Priority,
    Attack,
    Block,
    /// A pending prompt, by its `Decision` variant name.
    Prompt(&'static str),
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Priority => "priority",
            Kind::Attack => "attack",
            Kind::Block => "block",
            Kind::Prompt(p) => p,
        }
    }
}

/// The moves at one decision.
#[derive(Debug, Clone)]
pub struct Menu {
    pub kind: Kind,
    pub moves: Vec<GameAction>,
    /// A cap bound, or the answer space is open (free text): the list is
    /// not every legal move.
    pub truncated: bool,
}

/// The `Decision` variant's name.
pub fn prompt_name(d: &Decision) -> &'static str {
    match d {
        Decision::ChooseTarget { .. } => "ChooseTarget",
        Decision::ChooseMode { .. } => "ChooseMode",
        Decision::ChooseColor { .. } => "ChooseColor",
        Decision::Scry { .. } => "Scry",
        Decision::Discard { .. } => "Discard",
        Decision::SearchLibrary { .. } => "SearchLibrary",
        Decision::OptionalTrigger { .. } => "OptionalTrigger",
        Decision::PutOnLibrary { .. } => "PutOnLibrary",
        Decision::Mulligan { .. } => "Mulligan",
        Decision::ChooseCreatureType { .. } => "ChooseCreatureType",
        Decision::ChooseCreatureTypePair { .. } => "ChooseCreatureTypePair",
        Decision::NameCard { .. } => "NameCard",
        Decision::CoinFlip { .. } => "CoinFlip",
        Decision::DieRoll { .. } => "DieRoll",
        Decision::CommanderRedirect { .. } => "CommanderRedirect",
        Decision::CombatDamageOrder { .. } => "CombatDamageOrder",
        Decision::ChooseModes { .. } => "ChooseModes",
        Decision::Learn { .. } => "Learn",
        Decision::OrderTriggers { .. } => "OrderTriggers",
        Decision::DivideDamage { .. } => "DivideDamage",
        Decision::AssignCombatDamage { .. } => "AssignCombatDamage",
        Decision::ChooseLegendToKeep { .. } => "ChooseLegendToKeep",
        Decision::ChooseAmount { .. } => "ChooseAmount",
        Decision::ChooseOption { .. } => "ChooseOption",
        Decision::ChooseCards { .. } => "ChooseCards",
    }
}

/// The moves `seat` has right now, or `None` when it has nothing to decide
/// (not its priority, no prompt of its own, no declaration due).
pub fn menu(g: &GameState, seat: usize, w: &EvalWeights) -> Option<Menu> {
    if g.is_game_over() {
        return None;
    }
    if let Some(p) = &g.pending_decision {
        if p.acting_player() != seat {
            return None;
        }
        let (moves, truncated) = prompt_moves(g, &p.decision);
        return Some(Menu { kind: Kind::Prompt(prompt_name(&p.decision)), moves, truncated });
    }
    let attack = g.step == TurnStep::DeclareAttackers
        && g.attack_declarer() == seat
        && g.attacking().is_empty();
    let block = g.step == TurnStep::DeclareBlockers
        && g.may_declare_blocks(seat)
        && !g.attacking().is_empty()
        && !g.blockers_declared();
    if !attack && !block && g.player_with_priority() != seat {
        return None;
    }
    let e = puzzle::legal_actions(g, seat);
    let mut truncated = e.truncated;
    let mut moves: Vec<GameAction> = Vec::new();
    let mut keys: Vec<String> = Vec::new();
    let push = |a: GameAction, moves: &mut Vec<GameAction>, keys: &mut Vec<String>| {
        let k = key(g, seat, &a);
        if !keys.contains(&k) {
            keys.push(k);
            moves.push(a);
        }
    };
    // Pass first: a list cut at the cap must still carry it.
    let mut seeds: Vec<GameAction> = e.actions;
    // (`legal_actions` adds it last and truncates after, so a wide
    // position can drop it there.)
    seeds.retain(|a| !matches!(a, GameAction::PassPriority));
    if g.player_with_priority() == seat {
        seeds.insert(0, GameAction::PassPriority);
    }
    if attack {
        // The brute force swings at the player only; a planeswalker or a
        // battle is an attack target too.
        seeds.retain(|a| !matches!(a, GameAction::DeclareAttackers(_)));
        let (decls, cut) = attack_declarations(g, seat);
        truncated |= cut;
        seeds.extend(decls);
    }
    if block {
        // The brute force tries each blocker alone, so an attacker only two
        // can block (menace) was never offered (`Apex Witchstalker`, FRA).
        seeds.retain(|a| !matches!(a, GameAction::DeclareBlockers(_)));
        let (decls, cut) = block_declarations(g, seat);
        truncated |= cut;
        seeds.extend(decls);
    }
    if g.player_with_priority() == seat {
        seeds.extend(super::bot::cast_candidates(g, seat, w, None).into_iter().map(|(a, _)| a));
        seeds.extend(super::bot::main_phase_sink_action(g, seat, w));
    }
    if g.player_with_priority() == seat {
        seeds.extend(other_moves(g, seat));
        // The search's own arms in the window it runs (its alternative
        // targetings and X sizes are built there, not in the generators).
        if g.active_player_idx == seat
            && g.stack.is_empty()
            && matches!(g.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain)
        {
            seeds.extend(super::bot::main_phase_candidates_for_mcts(g, seat, w).into_iter().map(|(a, _)| a));
        }
    }
    // A combat step's declaration can also be made by passing priority (the
    // engine reads it as "none").
    if (attack || block) && g.would_accept(GameAction::PassPriority) {
        seeds.insert(0, GameAction::PassPriority);
    }
    for a in seeds {
        // The move as generated and every target / X / mode variant of it:
        // a generator's own pick stays even where the fan-out's filters
        // would not reach it.
        let fanned = fan_out(g, seat, &a);
        if accepts(g, seat, &a) {
            push(a, &mut moves, &mut keys);
        }
        for f in fanned {
            push(f, &mut moves, &mut keys);
        }
    }
    if moves.len() > MAX_MOVES {
        moves.truncate(MAX_MOVES);
        truncated = true;
    }
    let kind = if attack {
        Kind::Attack
    } else if block {
        Kind::Block
    } else {
        Kind::Priority
    };
    Some(Menu { kind, moves, truncated })
}

/// Moves no other generator offers: hand activations (cycling,
/// landcycling, discard abilities, foretell, plot, a face-down cast, an
/// MDFC's land back), lands from the graveyard, the library's top and exile,
/// equip on each creature, turning a face-down permanent up. Each kept only
/// if the engine accepts it.
fn other_moves(g: &GameState, seat: usize) -> Vec<GameAction> {
    let p = &g.players[seat];
    let mut cands: Vec<GameAction> = Vec::new();
    for c in p.hand.iter() {
        let id = c.id;
        cands.extend([
            GameAction::Cycle { card_id: id, x_value: None },
            GameAction::Landcycle { card_id: id },
            GameAction::ActivateDiscardAbility { card_id: id },
            GameAction::Foretell { card_id: id },
            GameAction::Plot { card_id: id },
            GameAction::CastFaceDown { card_id: id },
            GameAction::PlayLandBack(id),
        ]);
    }
    for c in p.graveyard.iter().filter(|c| c.definition.is_land()) {
        cands.push(GameAction::PlayLandFromGraveyard(c.id));
    }
    for c in p.library.last().into_iter().chain(g.exile.iter()).filter(|c| c.definition.is_land()) {
        cands.push(GameAction::PlayLand(c.id));
    }
    let creatures: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_creature()).map(|c| c.id).collect();
    for c in g.battlefield.iter().filter(|c| c.controller == seat) {
        if c.definition.card_types.contains(&crate::card::CardType::Artifact) && !c.definition.is_creature() {
            cands.extend(creatures.iter().map(|&t| GameAction::Equip { equipment: c.id, target: t }));
        }
        if c.face_down {
            cands.push(GameAction::TurnFaceUp { card_id: c.id });
        }
    }
    cands.into_iter().filter(|a| accepts(g, seat, a)).collect()
}

/// Whether the engine takes `a` from `seat`: [`GameState::would_accept`],
/// or — for a move that pauses on a cost prompt of its own (a landcycling
/// pick, a sacrifice to pay) — whether some answer to that prompt completes
/// it. `would_accept` refuses those for a bot seat (a paused cast the bot
/// can't finish deadlocked cube games), but they are legal moves and the
/// client's pilot makes them.
fn accepts(g: &GameState, seat: usize, a: &GameAction) -> bool {
    if g.would_accept(a.clone()) {
        return true;
    }
    let mut probe = g.clone();
    if probe.perform_action(a.clone()).is_err() {
        return false;
    }
    let Some(p) = probe.pending_decision.as_ref().filter(|p| p.resume.is_action_replay() && p.acting_player() == seat) else {
        return false;
    };
    let (answers, _) = prompt_moves(&probe, &p.decision);
    answers.into_iter().take(16).any(|ans| {
        let mut t = probe.clone();
        t.perform_action(ans).is_ok() && !t.pending_decision.as_ref().is_some_and(|p| p.resume.is_action_replay())
    })
}

/// Every attack declaration: each eligible creature at no target or at any
/// target it may attack (the player, an opposing planeswalker or battle),
/// the whole set engine-validated. Capped at eight creatures and
/// [`MAX_MOVES`] sets.
fn attack_declarations(g: &GameState, seat: usize) -> (Vec<GameAction>, bool) {
    use crate::game::{Attack, AttackTarget};
    let mut targets = vec![AttackTarget::Player(g.next_alive_seat(seat))];
    for c in g.battlefield.iter().filter(|c| c.controller != seat) {
        if c.definition.is_planeswalker() {
            targets.push(AttackTarget::Planeswalker(c.id));
        }
        if c.definition.card_types.contains(&crate::card::CardType::Battle) {
            targets.push(AttackTarget::Battle(c.id));
        }
    }
    let mut truncated = false;
    let mut options: Vec<Vec<Attack>> = Vec::new();
    for c in g.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_creature()) {
        let mine: Vec<Attack> = targets
            .iter()
            .map(|&target| Attack { attacker: c.id, target })
            .filter(|a| g.would_accept(GameAction::DeclareAttackers(vec![*a])))
            .collect();
        if !mine.is_empty() {
            options.push(mine);
        }
    }
    if options.len() > 8 {
        options.truncate(8);
        truncated = true;
    }
    let mut sets: Vec<Vec<Attack>> = vec![Vec::new()];
    for opts in &options {
        let mut next = Vec::new();
        for base in &sets {
            next.push(base.clone());
            for a in opts {
                let mut b = base.clone();
                b.push(*a);
                next.push(b);
            }
        }
        if next.len() > MAX_MOVES * 4 {
            next.truncate(MAX_MOVES * 4);
            truncated = true;
        }
        sets = next;
    }
    let decls = sets
        .into_iter()
        .map(GameAction::DeclareAttackers)
        .filter(|a| g.would_accept(a.clone()))
        .collect();
    (decls, truncated)
}

/// Every block declaration: each untapped creature on no attacker or on
/// any attacker it can block — alone, or with one other blocker (menace) —
/// the whole set engine-validated. Capped at eight blockers and
/// [`MAX_MOVES`] sets.
fn block_declarations(g: &GameState, seat: usize) -> (Vec<GameAction>, bool) {
    let mut truncated = false;
    let mut mine: Vec<CardId> = g
        .battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature() && !c.tapped)
        .map(|c| c.id)
        .collect();
    if mine.len() > 8 {
        mine.truncate(8);
        truncated = true;
    }
    let attackers: Vec<CardId> = g.attacking().iter().map(|a| a.attacker).collect();
    let ok = |v: Vec<(CardId, CardId)>| g.would_accept(GameAction::DeclareBlockers(v));
    let options: Vec<Vec<Option<CardId>>> = mine
        .iter()
        .map(|&b| {
            let mut v = vec![None];
            for &a in &attackers {
                if ok(vec![(b, a)]) || mine.iter().any(|&b2| b2 != b && ok(vec![(b, a), (b2, a)])) {
                    v.push(Some(a));
                }
            }
            v
        })
        .collect();
    let mut sets: Vec<Vec<(CardId, CardId)>> = vec![Vec::new()];
    for (i, opts) in options.iter().enumerate() {
        let mut next = Vec::new();
        for base in &sets {
            for o in opts {
                let mut b = base.clone();
                if let Some(a) = o {
                    b.push((mine[i], *a));
                }
                next.push(b);
            }
        }
        if next.len() > MAX_MOVES * 4 {
            next.truncate(MAX_MOVES * 4);
            truncated = true;
        }
        sets = next;
    }
    let decls = sets.into_iter().filter(|v| ok(v.clone())).map(GameAction::DeclareBlockers).collect();
    (decls, truncated)
}

/// The canonical key of a move: its debug text, card ids named, the
/// target dropped where the spell or ability takes none, and a prompt's
/// "engine default" answers spelled out.
pub fn key(g: &GameState, seat: usize, a: &GameAction) -> String {
    let mut a = a.clone();
    // Unordered answers compare as sets.
    match &mut a {
        GameAction::DeclareAttackers(v) => v.sort_by_key(|x| x.attacker),
        GameAction::DeclareBlockers(v) => v.sort(),
        GameAction::SubmitDecision(DecisionAnswer::Discard(v) | DecisionAnswer::Cards(v)) => v.sort(),
        GameAction::SubmitDecision(DecisionAnswer::Modes(v)) => v.sort(),
        _ => {}
    }
    if let Some(eff) = effect_of(g, seat, &a)
        && !eff.requires_target()
    {
        clear_targets(&mut a);
    }
    name_ids(g, &format!("{a:?}"))
}

/// `CardId(n)` → the card's name wherever it is.
fn name_ids(g: &GameState, text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("CardId(") {
        out.push_str(&rest[..i]);
        let tail = &rest[i + 7..];
        let Some(j) = tail.find(')') else {
            out.push_str(&rest[i..]);
            return out;
        };
        match tail[..j].parse::<u32>() {
            Ok(n) => out.push_str(card_name(g, CardId(n))),
            Err(_) => out.push_str(&rest[i..i + 7 + j + 1]),
        }
        rest = &tail[j + 1..];
    }
    out.push_str(rest);
    out
}

fn card_name(g: &GameState, id: CardId) -> &'static str {
    if let Some(c) = g.battlefield_find(id) {
        return c.definition.name;
    }
    g.players
        .iter()
        .flat_map(|p| p.hand.iter().chain(p.graveyard.iter()).chain(p.library.iter()))
        .chain(g.exile.iter())
        .find(|c| c.id == id)
        .map(|c| c.definition.name)
        .unwrap_or("?")
}

/// The effect whose targets a cast or an activation chooses: the card's
/// spell effect wherever the card is, an activated or loyalty ability's
/// effect on the battlefield. `None` for anything else.
fn effect_of(g: &GameState, seat: usize, a: &GameAction) -> Option<Effect> {
    let find_card = |id: CardId| {
        let p = &g.players[seat];
        p.hand
            .iter()
            .chain(p.graveyard.iter())
            .chain(g.exile.iter())
            .find(|c| c.id == id)
            .map(|c| c.definition.effect.clone())
    };
    match a {
        GameAction::ActivateAbility { card_id, ability_index, .. } => g
            .battlefield_find(*card_id)
            .and_then(|c| c.definition.activated_abilities.get(*ability_index))
            .map(|ab| ab.effect.clone()),
        GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. } => g.battlefield_find(*card_id).and_then(|c| {
            crate::game::effective_loyalty_abilities(c, &g.battlefield).get(*ability_index).map(|ab| ab.effect.clone())
        }),
        GameAction::SubmitDecision(_) | GameAction::PassPriority => None,
        _ => {
            let v = serde_json::to_value(a).ok()?;
            let obj = v.as_object()?.values().next()?.as_object()?;
            let id = obj.get("card_id")?.as_u64()?;
            find_card(CardId(id as u32))
        }
    }
}

fn clear_targets(a: &mut GameAction) {
    let Ok(mut v) = serde_json::to_value(&*a) else { return };
    let Some(obj) = v.as_object_mut().and_then(|m| m.values_mut().next()).and_then(|x| x.as_object_mut()) else { return };
    let mut changed = false;
    if obj.get("target").is_some_and(|t| !t.is_null()) {
        obj.insert("target".into(), serde_json::Value::Null);
        changed = true;
    }
    if obj.get("additional_targets").is_some_and(|t| t.as_array().is_some_and(|x| !x.is_empty())) {
        obj.insert("additional_targets".into(), serde_json::Value::Array(Vec::new()));
        changed = true;
    }
    if changed && let Ok(b) = serde_json::from_value(v) {
        *a = b;
    }
}

/// A cast or activation over every target combination, X value and mode
/// its effect allows, each one the engine accepts. Empty when `a` has none
/// of those fields to vary (the caller keeps `a` itself).
fn fan_out(g: &GameState, seat: usize, a: &GameAction) -> Vec<GameAction> {
    let Ok(v) = serde_json::to_value(a) else { return Vec::new() };
    let Some((variant, body)) = v.as_object().and_then(|m| m.iter().next()) else { return Vec::new() };
    let Some(body) = body.as_object() else { return Vec::new() };
    let has = |f: &str| body.contains_key(f);
    if !has("target") && !has("x_value") && !has("mode") {
        return Vec::new();
    }
    let Some(eff) = effect_of(g, seat, a) else { return Vec::new() };
    let card_id = body.get("card_id").and_then(|x| x.as_u64()).map(|n| CardId(n as u32));

    // Target combinations, slot 0 in `target` and the rest in
    // `additional_targets`; an optional slot may stay empty.
    let mut combos: Vec<Vec<Option<Target>>> = vec![Vec::new()];
    if has("target") && eff.requires_target() {
        let slots = eff.target_slot_count().max(1);
        for s in 0..slots as u8 {
            let Some(filter) = eff.target_filter_for_slot(s) else { break };
            let mut opts: Vec<Option<Target>> =
                g.legal_targets_for_filter(filter, eff.accepts_player_target(), seat, card_id).into_iter().map(Some).collect();
            if eff.target_slot_optional(s, None) || opts.is_empty() {
                opts.push(None);
            }
            let mut next = Vec::new();
            for base in &combos {
                for o in &opts {
                    // A later slot can't be filled once an earlier one is empty.
                    if o.is_some() && base.last().is_some_and(|b: &Option<Target>| b.is_none()) {
                        continue;
                    }
                    let mut c = base.clone();
                    c.push(o.clone());
                    next.push(c);
                }
            }
            next.truncate(MAX_TARGET_COMBOS);
            combos = next;
        }
    }
    let xs: Vec<Option<u32>> = if has("x_value") && x_cost(g, seat, a) {
        (0..=MAX_X).map(Some).collect()
    } else {
        vec![body.get("x_value").and_then(|x| x.as_u64()).map(|x| x as u32)]
    };
    let modes: Vec<Option<usize>> = match (&eff, has("mode")) {
        // No mode is a move too: the engine asks for it on resolution.
        (Effect::ChooseMode(v), true) => std::iter::once(None).chain((0..v.len()).map(Some)).collect(),
        _ => vec![body.get("mode").and_then(|x| x.as_u64()).map(|x| x as usize)],
    };
    let mut out = Vec::new();
    for c in &combos {
        for &x in &xs {
            for &m in &modes {
                let mut b = body.clone();
                if has("target") && !c.is_empty() {
                    b.insert("target".into(), serde_json::to_value(c[0].clone()).unwrap_or_default());
                    if has("additional_targets") {
                        let rest: Vec<Target> = c[1..].iter().flatten().cloned().collect();
                        b.insert("additional_targets".into(), serde_json::to_value(rest).unwrap_or_default());
                    }
                }
                if has("x_value") {
                    b.insert("x_value".into(), serde_json::to_value(x).unwrap_or_default());
                }
                if has("mode") {
                    b.insert("mode".into(), serde_json::to_value(m).unwrap_or_default());
                }
                let mut wrapped = serde_json::Map::new();
                wrapped.insert(variant.clone(), serde_json::Value::Object(b));
                if let Ok(act) = serde_json::from_value::<GameAction>(serde_json::Value::Object(wrapped))
                    && accepts(g, seat, &act)
                {
                    out.push(act);
                }
            }
        }
    }
    out
}

/// Whether the card a cast names has {X} in its cost.
fn x_cost(g: &GameState, seat: usize, a: &GameAction) -> bool {
    let Ok(v) = serde_json::to_value(a) else { return false };
    let Some(id) = v.as_object().and_then(|m| m.values().next()).and_then(|b| b.get("card_id")).and_then(|x| x.as_u64())
    else {
        return false;
    };
    let id = CardId(id as u32);
    let p = &g.players[seat];
    p.hand
        .iter()
        .chain(p.graveyard.iter())
        .chain(g.exile.iter())
        .find(|c| c.id == id)
        .is_some_and(|c| c.definition.cost.has_x())
}

// ── Prompts ────────────────────────────────────────────────────────────

/// Every answer to `d` (up to [`MAX_MOVES`]), and whether the list is cut.
fn prompt_moves(g: &GameState, d: &Decision) -> (Vec<GameAction>, bool) {
    let mut truncated = false;
    let ids = |v: &[(CardId, String)]| v.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    let answers: Vec<DecisionAnswer> = match d {
        Decision::ChooseTarget { legal, optional, .. } => {
            let mut v: Vec<DecisionAnswer> = legal.iter().cloned().map(DecisionAnswer::Target).collect();
            if *optional || legal.is_empty() {
                v.push(DecisionAnswer::DeclineTarget);
            }
            v
        }
        Decision::ChooseMode { num_modes, .. } => (0..*num_modes).map(DecisionAnswer::Mode).collect(),
        Decision::ChooseColor { legal, .. } => legal.iter().map(|c| DecisionAnswer::Color(*c)).collect(),
        Decision::Scry { cards, mode, .. } => {
            let c = ids(cards);
            match mode {
                ScryMode::Rearrange => perms(&c, c.len(), &mut truncated)
                    .into_iter()
                    .map(|p| DecisionAnswer::ScryOrder { kept_top: p, bottom: vec![] })
                    .collect(),
                ScryMode::OrderBottom => perms(&c, c.len(), &mut truncated)
                    .into_iter()
                    .map(|p| DecisionAnswer::ScryOrder { kept_top: vec![], bottom: p })
                    .collect(),
                // Scry and surveil: which cards stay on top, in what order;
                // the rest go down (or to the graveyard) in the given order.
                ScryMode::Scry | ScryMode::Surveil => {
                    let mut v = Vec::new();
                    for k in 0..=c.len() {
                        for top in perms(&c, k, &mut truncated) {
                            let bottom = c.iter().copied().filter(|x| !top.contains(x)).collect();
                            v.push(DecisionAnswer::ScryOrder { kept_top: top, bottom });
                        }
                    }
                    v
                }
            }
        }
        Decision::Discard { count, hand, .. } => {
            subsets(&ids(hand), *count as usize, *count as usize, &mut truncated).into_iter().map(DecisionAnswer::Discard).collect()
        }
        Decision::SearchLibrary { candidates, eligible, .. } => {
            let pool = eligible.clone().unwrap_or_else(|| ids(candidates));
            let mut v: Vec<DecisionAnswer> = pool.into_iter().map(|id| DecisionAnswer::Search(Some(id))).collect();
            v.push(DecisionAnswer::Search(None));
            v
        }
        Decision::OptionalTrigger { .. } | Decision::CommanderRedirect { .. } => {
            vec![DecisionAnswer::Bool(true), DecisionAnswer::Bool(false)]
        }
        Decision::CoinFlip { .. } => vec![DecisionAnswer::Bool(true), DecisionAnswer::Bool(false)],
        Decision::PutOnLibrary { count, hand, .. } => {
            perms(&ids(hand), *count, &mut truncated).into_iter().map(DecisionAnswer::PutOnLibrary).collect()
        }
        Decision::Mulligan { serum_powders, .. } => {
            let mut v = vec![DecisionAnswer::Keep, DecisionAnswer::TakeMulligan];
            v.extend(serum_powders.iter().map(|id| DecisionAnswer::SerumPowder(*id)));
            v
        }
        Decision::ChooseCreatureType { suggestions, excluded, .. } => {
            // Any creature type is legal; the suggestions are the ones that
            // matter on this board.
            truncated = true;
            suggestions.iter().filter(|t| !excluded.contains(t)).map(|t| DecisionAnswer::CreatureType(*t)).collect()
        }
        Decision::ChooseCreatureTypePair { suggestions, excluded, .. } => {
            truncated = true;
            let s: Vec<_> = suggestions.iter().filter(|t| !excluded.contains(t)).copied().collect();
            let mut v = Vec::new();
            for a in &s {
                for b in &s {
                    if a != b {
                        v.push(DecisionAnswer::CreatureTypePair(*a, *b));
                    }
                }
            }
            v
        }
        Decision::NameCard { suggestions, .. } => {
            truncated = true;
            suggestions.iter().cloned().map(DecisionAnswer::NamedCard).collect()
        }
        Decision::DieRoll { sides, rolled, .. } => {
            if (1..=*sides).contains(rolled) {
                vec![DecisionAnswer::DieRoll(*rolled)]
            } else {
                (1..=*sides).map(DecisionAnswer::DieRoll).collect()
            }
        }
        Decision::CombatDamageOrder { blockers, .. } => {
            let mut v = vec![DecisionAnswer::DamageOrder(Vec::new())];
            v.extend(perms(&ids(blockers), blockers.len(), &mut truncated).into_iter().map(DecisionAnswer::DamageOrder));
            v
        }
        Decision::ChooseModes { num_modes, count, default, .. } => {
            let all: Vec<u8> = (0..*num_modes as u8).collect();
            let mut v: Vec<DecisionAnswer> =
                subsets(&all, 1, *count, &mut truncated).into_iter().map(DecisionAnswer::Modes).collect();
            if !v.contains(&DecisionAnswer::Modes(default.clone())) {
                v.push(DecisionAnswer::Modes(default.clone()));
            }
            v
        }
        Decision::Learn { lessons, hand, .. } => {
            let mut v: Vec<DecisionAnswer> =
                lessons.iter().map(|(id, _)| DecisionAnswer::Learn(LearnChoice::FetchLesson(*id))).collect();
            v.extend(hand.iter().map(|(id, _)| DecisionAnswer::Learn(LearnChoice::Rummage { discard: *id })));
            v.push(DecisionAnswer::Learn(LearnChoice::Decline));
            v
        }
        Decision::OrderTriggers { triggers, .. } => {
            let mut v = vec![DecisionAnswer::TriggerOrder(Vec::new())];
            v.extend(perms(&ids(triggers), triggers.len(), &mut truncated).into_iter().map(DecisionAnswer::TriggerOrder));
            v
        }
        Decision::DivideDamage { total, targets, .. } => {
            compositions(*total, targets.len(), &mut truncated).into_iter().map(DecisionAnswer::DamageDivision).collect()
        }
        Decision::AssignCombatDamage { attacker_power, blockers, .. } => {
            // The engine's default (lethal down the order) plus every split
            // of the power over the blockers.
            let mut v = vec![DecisionAnswer::CombatDamageAssignment(Vec::new())];
            let parts = compositions_with_zero(*attacker_power, blockers.len(), &mut truncated);
            v.extend(parts.into_iter().map(|p| {
                DecisionAnswer::CombatDamageAssignment(blockers.iter().map(|(id, _, _)| *id).zip(p).collect())
            }));
            v
        }
        Decision::ChooseLegendToKeep { duplicates, .. } => {
            duplicates.iter().map(|(id, _)| DecisionAnswer::KeptLegend(*id)).collect()
        }
        Decision::ChooseAmount { max, .. } => (0..=*max).map(DecisionAnswer::Amount).collect(),
        Decision::ChooseOption { options, .. } => (0..options.len() as u32).map(DecisionAnswer::Amount).collect(),
        Decision::ChooseCards { candidates, min, max, eligible, .. } => {
            let pool = eligible.clone().unwrap_or_else(|| ids(candidates));
            subsets(&pool, *min as usize, *max as usize, &mut truncated).into_iter().map(DecisionAnswer::Cards).collect()
        }
    };
    let mut moves: Vec<GameAction> = answers.into_iter().map(GameAction::SubmitDecision).collect();
    if moves.len() > MAX_MOVES {
        moves.truncate(MAX_MOVES);
        truncated = true;
    }
    let _ = g;
    (moves, truncated)
}

/// Ordered selections of `k` from `items`, up to [`MAX_MOVES`].
fn perms<T: Copy + PartialEq>(items: &[T], k: usize, truncated: &mut bool) -> Vec<Vec<T>> {
    let mut out = Vec::new();
    let mut cur = Vec::with_capacity(k);
    fn rec<T: Copy + PartialEq>(items: &[T], k: usize, cur: &mut Vec<T>, out: &mut Vec<Vec<T>>, truncated: &mut bool) {
        if out.len() >= MAX_MOVES {
            *truncated = true;
            return;
        }
        if cur.len() == k {
            out.push(cur.clone());
            return;
        }
        for &x in items {
            if !cur.contains(&x) {
                cur.push(x);
                rec(items, k, cur, out, truncated);
                cur.pop();
            }
        }
    }
    if k <= items.len() {
        rec(items, k, &mut cur, &mut out, truncated);
    }
    out
}

/// Unordered subsets of `items` of size `lo..=hi`, up to [`MAX_MOVES`].
fn subsets<T: Copy>(items: &[T], lo: usize, hi: usize, truncated: &mut bool) -> Vec<Vec<T>> {
    let mut out = Vec::new();
    fn rec<T: Copy>(items: &[T], start: usize, lo: usize, hi: usize, cur: &mut Vec<T>, out: &mut Vec<Vec<T>>, truncated: &mut bool) {
        if out.len() >= MAX_MOVES {
            *truncated = true;
            return;
        }
        if cur.len() >= lo {
            out.push(cur.clone());
        }
        if cur.len() == hi {
            return;
        }
        for i in start..items.len() {
            cur.push(items[i]);
            rec(items, i + 1, lo, hi, cur, out, truncated);
            cur.pop();
        }
    }
    rec(items, 0, lo, hi.min(items.len()), &mut Vec::new(), &mut out, truncated);
    out
}

/// Splits of `total` into `n` parts of at least one each.
fn compositions(total: u32, n: usize, truncated: &mut bool) -> Vec<Vec<u32>> {
    if n == 0 || (total as usize) < n {
        return vec![Vec::new()];
    }
    compositions_with_zero(total - n as u32, n, truncated).into_iter().map(|v| v.into_iter().map(|x| x + 1).collect()).collect()
}

/// Splits of `total` into `n` parts of zero or more.
fn compositions_with_zero(total: u32, n: usize, truncated: &mut bool) -> Vec<Vec<u32>> {
    let mut out = Vec::new();
    fn rec(left: u32, n: usize, cur: &mut Vec<u32>, out: &mut Vec<Vec<u32>>, truncated: &mut bool) {
        if out.len() >= MAX_MOVES {
            *truncated = true;
            return;
        }
        if cur.len() + 1 == n {
            cur.push(left);
            out.push(cur.clone());
            cur.pop();
            return;
        }
        for x in 0..=left {
            cur.push(x);
            rec(left - x, n, cur, out, truncated);
            cur.pop();
        }
    }
    if n == 0 {
        return vec![Vec::new()];
    }
    rec(total, n, &mut Vec::new(), &mut out, truncated);
    out
}

// ── The coverage census ────────────────────────────────────────────────

/// One decision the pilot made, against the list.
#[derive(Debug, Clone)]
pub struct CensusRow {
    pub kind: Kind,
    pub moves: usize,
    pub truncated: bool,
    /// The pilot's move is on the list (by [`key`]).
    pub covered: bool,
    /// The pilot's move: its variant and key, for the misses list.
    pub variant: String,
    pub chosen: String,
    /// Prompt answers on the list that the engine refused on a clone.
    pub rejected: usize,
}

/// Play a game with `pilot` on both seats and check every decision either
/// seat makes against its list.
pub fn census_game(template: &GameState, pilot: super::MctsConfig, seed: u64) -> Vec<CensusRow> {
    use super::bot::Bot;
    let w = pilot.weights;
    let mut g = super::action_gap::start_game(template, &w, seed);
    let mut bots = [super::MctsBot::new(pilot), super::MctsBot::new(pilot)];
    let mut out = Vec::new();
    let mut fuel = 6000u32;
    while fuel > 0 && !g.is_game_over() && g.turn_number <= 30 {
        fuel -= 1;
        let mut acted = false;
        for (seat, bot) in bots.iter_mut().enumerate() {
            let m = menu(&g, seat, &w);
            let Some(a) = bot.next_action(&g, seat) else { continue };
            let before = g.clone();
            if g.perform_action(a.clone()).is_err() {
                continue;
            }
            acted = true;
            if let Some(m) = m {
                let chosen = key(&before, seat, &a);
                let covered = m.moves.iter().any(|x| key(&before, seat, x) == chosen);
                let rejected = if matches!(m.kind, Kind::Prompt(_)) {
                    m.moves.iter().take(32).filter(|x| before.clone().perform_action((*x).clone()).is_err()).count()
                } else {
                    0
                };
                let variant = format!("{a:?}").split([' ', '(', '{']).next().unwrap_or("").to_string();
                let variant = match &a {
                    GameAction::SubmitDecision(d) => {
                        format!("SubmitDecision::{}", format!("{d:?}").split([' ', '(', '{']).next().unwrap_or(""))
                    }
                    _ => variant,
                };
                out.push(CensusRow {
                    kind: m.kind,
                    moves: m.moves.len(),
                    truncated: m.truncated,
                    covered,
                    variant,
                    chosen,
                    rejected,
                });
            }
            break;
        }
        if !acted && g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinatorics_count_what_they_should() {
        let mut t = false;
        assert_eq!(perms(&[1, 2, 3], 2, &mut t).len(), 6);
        assert_eq!(subsets(&[1, 2, 3, 4], 2, 2, &mut t).len(), 6);
        assert_eq!(subsets(&[1, 2, 3], 0, 3, &mut t).len(), 8);
        assert_eq!(compositions(5, 2, &mut t), vec![vec![1, 4], vec![2, 3], vec![3, 2], vec![4, 1]]);
        assert_eq!(compositions_with_zero(2, 2, &mut t).len(), 3);
        assert!(!t);
    }

    /// A two-slot spell fans out over its second slot: Burrog Barrage
    /// ("target creature you control … deals damage … to up to one target
    /// creature an opponent controls") reaches the opposing creature only
    /// through `additional_targets`, which the brute force never fills.
    #[test]
    fn a_two_target_bite_fans_out_over_its_second_slot() {
        let mut g = crate::game::two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        for _ in 0..2 {
            let id = g.add_card_to_battlefield(0, crate::catalog::forest());
            g.clear_sickness(id);
        }
        let barrage = g.add_card_to_hand(0, crate::catalog::burrog_barrage());
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        let them = g.add_card_to_battlefield(1, crate::catalog::grizzly_bears());
        let m = menu(&g, 0, &EvalWeights::default()).expect("priority");
        let bite = m.moves.iter().any(|a| {
            matches!(a, GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), additional_targets, .. }
                if *card_id == barrage && *t == bear && additional_targets == &vec![Target::Permanent(them)])
        });
        assert!(bite, "no bite in {:#?}", m.moves);
    }

    /// An opposing planeswalker is an attack target: the brute force only
    /// ever swung at the player, so the client's attacks on Ral Zarek were
    /// off the list (the census's first misses).
    #[test]
    fn attacks_on_a_planeswalker_are_on_the_list() {
        use crate::card::{CardDefinition, CardType, CounterType};
        use crate::game::{Attack, AttackTarget};
        let mut g = crate::game::two_player_game();
        for seat in 0..2 {
            for _ in 0..5 {
                g.add_card_to_library(seat, crate::catalog::island());
            }
        }
        let bear = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
        g.clear_sickness(bear);
        let walker = g.add_card_to_battlefield(
            1,
            CardDefinition { name: "Walker", card_types: vec![CardType::Planeswalker], ..Default::default() },
        );
        if let Some(c) = g.battlefield_find_mut(walker) {
            c.add_counters(CounterType::Loyalty, 3);
        }
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        let m = menu(&g, 0, &EvalWeights::default()).expect("attack");
        assert_eq!(m.kind, Kind::Attack);
        let at_walker = GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Planeswalker(walker) }]);
        let at_player = GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]);
        assert!(m.moves.contains(&at_walker) && m.moves.contains(&at_player), "{:#?}", m.moves);
        assert!(m.moves.contains(&GameAction::DeclareAttackers(Vec::new())));
    }

    /// A menace attacker can be blocked by two: the brute force tried each
    /// blocker alone and so never offered blocking it at all.
    #[test]
    fn a_double_block_on_a_menace_attacker_is_on_the_list() {
        use crate::card::{CardDefinition, CardType, Keyword};
        use crate::game::{Attack, AttackTarget};
        let mut g = crate::game::two_player_game();
        for seat in 0..2 {
            for _ in 0..5 {
                g.add_card_to_library(seat, crate::catalog::island());
            }
        }
        let body = |name: &'static str, kw: Vec<Keyword>| CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: 2,
            toughness: 2,
            keywords: kw,
            ..Default::default()
        };
        let b1 = g.add_card_to_battlefield(0, body("Guard A", vec![]));
        let b2 = g.add_card_to_battlefield(0, body("Guard B", vec![]));
        let brute = g.add_card_to_battlefield(1, body("Brute", vec![Keyword::Menace]));
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.clear_sickness(brute);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: brute, target: AttackTarget::Player(0) }]))
            .expect("attack");
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        let m = menu(&g, 0, &EvalWeights::default()).expect("block");
        assert_eq!(m.kind, Kind::Block);
        let double = key(&g, 0, &GameAction::DeclareBlockers(vec![(b1, brute), (b2, brute)]));
        assert!(m.moves.iter().any(|a| key(&g, 0, a) == double), "{:#?}", m.moves);
        assert!(!m.moves.contains(&GameAction::DeclareBlockers(vec![(b1, brute)])), "a lone block on menace is illegal");
    }
}
